# TASK 03 — Definir threat model, identidade, trust e capabilities

Status: decisão de segurança concluída; implementação de identidade, pairing e autorização fica para as tasks seguintes

## Objetivo

Estabelecer a política de segurança que deve anteceder qualquer comunicação real do Pulse. A task define o que é uma identidade verificável, como um pairing cria trust, como uma revogação interrompe operações e como cada capability autoriza somente o recurso e a direção aprovados.

Esta task é documental e de decisão. Ela não adiciona rede, criptografia de produção, armazenamento, comandos Tauri, eventos IPC ou integração com os mocks.

## Estado atual

- O produto é local-first, mas a rede local não é considerada confiável por padrão; a TASK 11 implementou somente discovery mDNS e candidatos transitórios. Pairing, identidade, trust e revogação ainda não existem (`PRODUCT.md:5-7,27-33,56-63`).
- O contrato canônico separa candidato, presença, pairing, trust e capability (`src/types/index.ts:86-179`, `src-tauri/src/domain/mod.rs:46-297`). Os tipos atuais são modelos puros, não uma implementação de autenticação ou autorização.
- `DiscoveryCandidate` carrega nome, endpoint e capabilities anunciadas sem prova de identidade (`src/types/index.ts:96-109`). A TASK 02 decidiu mDNS/DNS-SD para discovery e QUIC v1 via `quinn`, mas também deixou explícito que anúncio e sessão QUIC não concedem confiança (`docs/tasks/TASK-02-discovery-transporte-e-conexao-local.md:47-88,122-145`).
- `PresentedIdentity` possui apenas dados de apresentação opcionais (`src/types/index.ts:133-138`); ainda não há registro de chave pública, prova assinada, fingerprint verificável ou política de armazenamento de segredo.
- `CapabilityGrant` já separa decisão de autorização de `CapabilityInfo.available`, mas a aplicação ainda não consulta grants e os stores Vue continuam mockados (`src/types/index.ts:164-179`, `src/stores/`).
- `TrustRelationship.pairingSessionId` é opcional e os tipos não exigem estruturalmente uma prova de chave ou pairing confirmado para o estado `trusted` (`src/types/index.ts:154-162`, `src-tauri/src/domain/mod.rs:268-277`). Essa exigência fica como invariante de serviço até que os contratos de implementação a representem.
- O runtime Rust registra `greet`, storage e o browse mDNS da TASK 11; ainda não há sockets de transporte, listeners de produto, keyring ou comandos de produto (`src-tauri/src/lib.rs`, `src-tauri/Cargo.toml`, `SYSTEM-DESIGN.md:78-105`).
- A capability Tauri `core:default` é uma permissão do shell e não deve ser confundida com capability do Pulse entre peers (`src-tauri/capabilities/default.json`, `SYSTEM-DESIGN.md:102-104,148-161`).

## Brainstorm

### Ativos e fronteiras de confiança

Os ativos que exigem proteção são:

- chave privada da identidade local e qualquer segredo de sessão;
- chave pública, fingerprint, relação de trust, decisões de capability e bloqueios de revogação;
- arquivos, conteúdo de Clipboard, texto, links, metadados de mídia e resultados de comandos;
- caminhos locais, diretórios de destino, histórico de decisões e notificações;
- integridade, origem e correlação de mensagens entre peers.

As fronteiras são:

1. **LAN e discovery:** ambiente não confiável. Nome, IP, porta, plataforma, TXT, `CandidateId` e capabilities anunciadas são apenas dados de seleção.
2. **Transporte QUIC/TLS:** fornece confidencialidade, integridade e negociação de protocolo, mas não decide se o peer é um dispositivo confiável nem se uma operação está autorizada.
3. **Domínio Rust:** mantém identidade, trust, grants, sessões e validação antes de efeitos locais.
4. **Bridge Tauri:** recebe intenções da UI e expõe somente estado e erros necessários; não entrega chaves privadas à UI.
5. **Recursos locais:** arquivos, Clipboard, mídia, notificações e comandos são protegidos separadamente por capability e por validação específica.

### Adversários considerados

| ID | Adversário/cenário | Objetivo | Limite da proteção |
| --- | --- | --- | --- |
| A1 | Observador passivo na LAN | Ler conteúdo ou metadados de sessão | O canal deve proteger conteúdo; volume, presença e alguns metadados de rede podem continuar observáveis. |
| A2 | Atacante ativo na LAN | Spoofar discovery, fazer MITM, replayar pairing ou forçar downgrade | A autenticação e a confirmação humana devem impedir confiança silenciosa; disponibilidade não é garantida. |
| A3 | Candidato malicioso ou peer não pareado | Enviar payloads, solicitar recursos ou provocar efeitos locais | O candidato só pode participar de identificação/pairing limitado e sujeito a limites. |
| A4 | Peer pareado tentando extrapolar autorização | Ler, escrever, transferir ou executar além do grant | Cada operação deve verificar trust, capability, direção, estado e parâmetros no dispositivo que possui o recurso. |
| A5 | Peer com chave privada comprometida | Representar a identidade antiga e acessar grants existentes | Revogação local deve bloquear a identidade; rotação cria novo `DeviceId` e exige novo pairing. |
| A6 | Processo malicioso no mesmo usuário/sessão | Capturar comandos da bridge ou pedir efeitos em nome da UI | A implementação deve reduzir a superfície da bridge e validar intenções; comprometimento total do usuário/OS fica fora do modelo. |
| A7 | Rede indisponível, firewall ou flood | Impedir discovery, conexão ou progresso | O sistema deve falhar fechado e observavelmente, mas não promete disponibilidade contra DoS. |

Ficam fora de escopo: cloud, relay e identidade de uma autoridade remota; recuperação de um sistema operacional ou conta de usuário comprometidos; segurança física após desbloqueio do dispositivo; e consentimento do usuário obtido por fraude fora dos controles de confirmação do Pulse. O produto deve, ainda assim, tornar a decisão de confiança legível e mostrar fingerprint, origem, destino e consequência.

### Ameaças e controles obrigatórios

| Ameaça | Controle de política | Resultado esperado |
| --- | --- | --- |
| Candidato falsifica nome, IP, porta ou TXT | Tratar discovery como não autenticado; usar a chave pública autenticada como âncora | Nenhum candidato vira `trusted` ou ganha capability. |
| MITM durante pairing | Prova assinada por ambas as identidades, nonces frescos e comparação do mesmo código curto nos dois dispositivos | Divergência de transcript, identidade ou código aborta a sessão sem criar trust. |
| Replay de pedido, confirmação ou comando | `PairingSessionId` único, nonce aleatório, expiração curta, sessão de uso único, sequência/correlação e rejeição de duplicatas | Mensagens antigas não alteram trust nem repetem efeito local. |
| Downgrade de versão, algoritmo ou capability | Versão mínima, ALPN/protocolo e conjunto negociado vinculados ao transcript autenticado | Versão desconhecida ou incompatível falha fechada; não há fallback silencioso. |
| Certificado TLS autoassinado aceito sem verificar | A aplicação deve validar a identidade apresentada contra a chave pareada; nunca usar “aceitar qualquer certificado” em produto | QUIC estabelecido não é suficiente para abrir stream de recurso. |
| Peer confiável usa direção errada | Grant indexado por dispositivo, chave, direção e recurso; verificação no owner do recurso | `files.send` não autoriza `files.receive`, e assim por diante. |
| Payload malformado ou grande | Parsing estrito, limites de tamanho/frequência, timeouts, backpressure e validação antes de efeito | Entrada inválida gera erro observável e não toca arquivos, Clipboard ou comandos. |
| Caminho ou link malicioso | Caminhos recebidos não são comandos; normalização, política de destino e allowlist pertencem ao serviço de arquivos; link não abre automaticamente | Não há traversal, sobrescrita silenciosa ou execução por deserialização. |
| Peer revogado tenta reconectar | Bloqueio local por identidade, fechamento de sessões e invalidação de resumption/tickets | Offline, erro de rede e revogado permanecem estados distintos; revogado não volta por reaparecer. |
| Segredo vaza em log, UI ou config | Chave privada somente no armazenamento seguro do SO e em memória protegida durante uso | A bridge e a UI recebem metadados mínimos, nunca material secreto. |

## Decisões

### 1. Identidade local e identificador do dispositivo

- Cada instalação gera uma identidade local na primeira inicialização que consiga persistir segredo com segurança. A identidade é um par de assinatura Ed25519 gerado por CSPRNG; a chave privada nunca é enviada à UI, ao discovery ou ao peer.
- A chave pública é a âncora criptográfica da identidade. `DeviceId` é uma referência opaca estável derivada do digest versionado da chave pública, com domínio `pulse/device-id/v1`; nome, plataforma, MAC, IP, porta, hostname, `CandidateId` e fingerprint curto não são identidade.
- A identidade publicada para autenticação contém, no mínimo, versão do formato, algoritmo, chave pública e `DeviceId`. Nome e plataforma são metadados apresentados e só podem ser tratados como autenticados quando vierem ligados a uma prova válida.
- O fingerprint completo é derivado da chave pública e o fingerprint curto/código de autenticação serve apenas para comparação humana. O código curto nunca substitui a chave pública nem vira segredo reutilizável.
- Rotação ou perda da chave que muda a chave pública cria uma nova identidade/`DeviceId` e exige novo pairing. Não haverá migração silenciosa de trust baseada em nome ou endpoint.
- A escolha de Ed25519 para assinatura, X25519/segredo efêmero para acordo de chaves quando necessário e HKDF para derivação versionada fica registrada como direção criptográfica. A implementação deve preferir primitivas de biblioteca revisada e não escrever criptografia própria.

### 2. Armazenamento de segredos

- A chave privada da identidade, chaves privadas de transporte persistidas (se a implementação vier a usar alguma) e material equivalente devem usar o armazenamento seguro nativo do sistema. No Linux, o backend inicial deve mirar Secret Service; o detalhe de crate/adaptador fica para a implementação e a TASK 04.
- O armazenamento seguro deve ser localizado por atributos estáveis, tolerar keyring bloqueado e não depender de caminho de arquivo gravável pela aplicação. O segredo não deve aparecer em logs, mensagens de erro, histórico, payload da bridge, dump de estado ou fixtures.
- Chave pública, fingerprint, `DeviceId`, trust, grants, timestamps e motivos não secretos podem ser persistidos pelo mecanismo definido na TASK 04.
- Sem keyring disponível, bloqueado ou corrompido, o Pulse não cria uma identidade estável nem oferece pairing/recursos autorizados. Não existe fallback de chave privada em texto puro, arquivo de configuração, Clipboard ou variável de ambiente.
- A implementação deve limitar a duração do segredo em memória e usar tipos/rotinas de zeroização quando a biblioteca escolhida fornecer essa garantia. Isso reduz exposição acidental, mas não protege contra comprometimento total do processo/OS.

### 3. Pairing autenticado

O pairing é uma sessão temporária, explícita e bilateral:

1. O usuário seleciona um candidato descoberto. Discovery pode sugerir um peer, mas não prova identidade nem inicia trust.
2. O Pulse cria uma `PairingSession` com expiração de 2 minutos, nonce local aleatório, uso único e limite de três confirmações inválidas. Estados `requested`, `awaiting-confirmation`, `rejected`, `expired`, `canceled` e `failed` não criam trust.
3. Os lados trocam identidades públicas, nonces, papéis, `PairingSessionId`, versão do modelo/protocolo e parâmetros solicitados. Cada lado prova posse da própria chave por assinatura do transcript canônico, com domínio separado para pairing.
4. Os dois dispositivos calculam um código de autenticação curto, de oito dígitos agrupados para leitura, a partir do transcript completo. Cada usuário deve confirmar que o código visto no outro lado é o mesmo e que o nome/fingerprint correspondem ao dispositivo pretendido.
5. Somente após as confirmações válidas dos dois lados a sessão passa a `confirmed` e a relação local passa de `unpaired` para `trusted`. A decisão registra sessão, fingerprint, momento e origem da aprovação.

O transcript deve incluir identidade, papéis, nonces, sessão, versões e parâmetros negociados para impedir que a comparação autentique apenas uma parte do handshake. Repetição, divergência, expiração, peer errado ou confirmação unilateral encerram a sessão sem conceder acesso. O mecanismo exato de transporte da prova e do código fica para as TASKS 16 e 20, mas nenhum desenho futuro pode aceitar certificado arbitrário, confiança por primeiro uso ou pairing silencioso.

O envelope de mensagens definido depois deve carregar, no mínimo, `requestId`, nonce ou sequência monotônica, epoch da sessão, origem, destino, capability pretendida e expiração. O receptor deve rejeitar duplicatas, mensagens fora da janela, origem/destino incompatíveis e requisições sem vínculo com uma sessão autenticada.

### 4. Canal seguro e vínculo com trust

- QUIC v1 usa TLS 1.3 para confidencialidade, integridade e chaves efêmeras de sessão. O serviço deve validar a identidade do peer contra a chave pareada e exigir autenticação mútua no nível apropriado; não confiar em uma CA pública para afirmar que um dispositivo local é do usuário.
- A identidade de longo prazo assina a prova de identidade; chaves efêmeras de transporte não substituem essa identidade. A mesma chave não deve ser reutilizada indiscriminadamente como chave de assinatura e de acordo de chaves.
- ALPN, versão mínima, modelo e parâmetros de segurança devem ser fixados ou autenticados no handshake. Falha de versão, identidade, certificado, assinatura, transcript ou capability encerra a sessão.
- 0-RTT/resumption não pode transportar operações com efeitos, pedidos de capability, confirmações de pairing, comandos, escritas de Clipboard ou transferências. Esses fluxos exigem uma sessão autenticada e anti-replay; resumption só poderá ser habilitado depois de uma decisão específica.
- Uma sessão QUIC estabelecida não cria trust e não abre recursos automaticamente. A ordem de autorização é: identidade autenticada → trust `trusted` → capability `granted` para o dispositivo, recurso e direção → validação da operação e, quando aplicável, aprovação por ação.

Na primeira versão, a persistência de resumption/tickets deve permanecer desabilitada. Se for habilitada depois, o ticket terá de respeitar a epoch de revogação e nunca poderá autorizar operação por si só.

### 5. Trust, perda de confiança e revogação

- `unpaired`, `trusted` e `revoked` continuam sendo estados de trust, independentes de `Presence`. `offline`, `stale`, `transport-blocked` e candidato expirado nunca alteram trust por si só.
- A revogação local é imediata: marca a relação como `revoked`, revoga os grants associados, fecha sessões ativas, invalida resumption/tickets e adiciona a identidade ao bloqueio local. O histórico mantém momento, origem e motivo sem preservar segredo ou conteúdo sensível. O motivo deve distinguir, no mínimo, `user-revoked` de `key-compromised`.
- Um peer revogado não pode iniciar nova sessão autorizada, mesmo com o mesmo nome, endpoint ou anúncio. Reaparecer na LAN não desfaz revogação.
- Uma ação local explícita de “parear novamente” pode mover a relação para `unpaired` e iniciar uma nova `PairingSession`; a confirmação deve ocorrer novamente. Essa transição não é executada por heartbeat, discovery, mensagem remota ou simples reconexão.
- Se houver suspeita de comprometimento da chave do peer, a identidade antiga permanece revogada. Uma nova chave exige novo `DeviceId`, nova apresentação e novo pairing; não há prova de continuidade baseada em metadados.
- Uma mensagem autenticada de auto-revogação do peer pode reduzir permissões e encerrar sessões, mas uma mensagem de rede não confiável nunca pode conceder, restaurar ou alterar trust local.
- Perda/corrupção da identidade local falha fechada. Recuperação, reset local e política de remoção de dados pertencem à TASK 04/15 e não podem importar automaticamente uma chave de backup sem decisão explícita.

### 6. Política de capabilities

`available` é suporte anunciado pelo peer e não é autorização. A ausência de grant equivale a negar. Todo grant é específico para um `DeviceId`, recurso, direção e versão de política; pairing não concede todos os recursos.

| Capability | Direção do efeito local | Padrão | Regra adicional |
| --- | --- | --- | --- |
| `files.send` | Este dispositivo → peer | Negada até pedido/aprovação | Cada envio ainda valida itens, tamanho, destino e aprovação da transferência. |
| `files.receive` | Peer → este dispositivo | Negada até pedido/aprovação | Grant não autoriza sobrescrita; cada recebimento passa por destino e conflito. |
| `clipboard.read` | Peer lê Clipboard local | Negada | Nunca habilitar sincronização contínua por implicação; conteúdo pode ser omitido pela política local. |
| `clipboard.write` | Peer escreve Clipboard local | Negada | Exige consentimento explícito e validação de tamanho/tipo; nunca executar conteúdo recebido. |
| `text.send` | Este dispositivo → peer | Negada até envio explícito | O usuário escolhe o conteúdo e o destino; não é um canal privilegiado. |
| `links.send` | Este dispositivo → peer | Negada até envio explícito | Receber um link nunca abre navegador ou executa ação automaticamente. |
| `media.read` | Peer lê estado de mídia local | Negada | Estado observado não concede `media.control`. |
| `media.control` | Peer controla mídia local | Negada | Ações são allowlistadas, parâmetros fechados e auditáveis; não há shell. |
| `notifications.receive` | Este dispositivo → peer recebe avisos | Negada | Copy e severidade são sanitizados; entrega local não prova sucesso remoto. |
| `commands.execute` | Peer solicita ação local | Negada | Alto risco: somente comandos enumerados, parâmetros validados, aprovação por ação e registro de resultado. |

As decisões de capability usam `requested`, `granted`, `denied` e `revoked`. `denied` não significa offline; `revoked` não significa ausência de suporte. O dispositivo que possui o recurso decide o grant correspondente; o peer não pode conceder uma permissão em nome do owner. Revogar trust revoga todos os grants; revogar um grant não revoga trust nem outros recursos.

`commands.execute` não representa shell, script, caminho arbitrário, processo, código ou injeção de input. A allowlist inicial deve permanecer limitada às ações canônicas do domínio (`device.ping` e controles de mídia definidos em `src/types/index.ts:346-375` e `src-tauri/src/domain/mod.rs:586-705`).

### 7. Auditoria e eventos de segurança

Falhas de segurança não podem desaparecer em um erro genérico. O contrato futuro deve conseguir registrar, com ID, origem, momento e motivo não sensível, pelo menos: pairing recusado/expirado/falho, identidade não verificada, downgrade rejeitado, replay descartado, trust revogado, capability negada/revogada e comando fora da allowlist. Esses fatos podem alimentar histórico e diagnóstico sem incluir chave, token, conteúdo de Clipboard, caminho completo ou payload bruto.

O vocabulário de resultado deve distinguir solicitação aceita, operação em execução e efeito confirmado. A confirmação de um pedido remoto não é sucesso do comando; a operação só termina quando o peer ou adaptador local confirmar o resultado.

### 8. Estado textual e UX de segurança

Antes de uma confirmação, a UI deve exibir origem, destino, nome apresentado, plataforma, fingerprint/código, capability ou ação solicitada, consequência e expiração. O texto deve distinguir:

- candidato descoberto de dispositivo confiável;
- sessão aguardando confirmação de trust concedido;
- peer offline de peer revogado;
- capability disponível de capability concedida;
- pedido aceito de operação concluída.

Nenhum ponto colorido, nome, endpoint, status online ou resposta de transporte pode ser usado como prova única. A UX detalhada de pairing, trust e grants pertence à TASK 19; esta política é o limite que a UX deve respeitar.

## Alternativas rejeitadas

| Alternativa | Decisão | Motivo |
| --- | --- | --- |
| Confiar no primeiro uso (TOFU) sem confirmação | Rejeitada | Um MITM na primeira conexão fixa uma chave falsa sem que o usuário perceba. |
| Usar nome, IP, MAC, porta, hostname ou `CandidateId` como identidade | Rejeitada | São falsificáveis, transitórios ou mudam com a rede. |
| Auto-trust após mDNS ou handshake QUIC | Rejeitada | Discovery e transporte atestam alcance/canal, não intenção do usuário nem autorização de recurso. |
| Aceitar qualquer certificado autoassinado | Rejeitada | TLS teria confidencialidade sem autenticação do peer. |
| Senha/PIN estático compartilhado | Rejeitada para v1 | É reutilizável, exige retenção de segredo e não vincula a confirmação ao transcript e às chaves. |
| CA/cloud/relay como autoridade de pairing | Rejeitada para v1 | Contradiz local-first e introduz dependência e superfície fora do escopo. |
| Conceder todas as capabilities ao parear | Rejeitada | Viola menor privilégio e torna trust equivalente a autorização operacional. |
| Criptografia própria de payload por cima de QUIC | Rejeitada como requisito | Duplica primitivas e aumenta risco; usar TLS 1.3/QUIC e biblioteca revisada, adicionando somente binding/protocolo necessário. |
| Chave privada em JSON/configuração/logs | Rejeitada | Expõe identidade e facilita cópia sem passar pelo controle do sistema operacional. |

## Plano de implementação

Esta task não implementa os itens abaixo; ela define as regras que as tasks responsáveis deverão cumprir.

1. TASK 04 deve definir tabelas persistidas, migração, retenção, remoção e recuperação para identidade pública, trust, grants, bloqueios e histórico não sensível.
2. TASK 15 deve gerar/carregar a identidade local, derivar `DeviceId`, integrar o armazenamento seguro e tratar perda/corrupção sem expor a chave à UI.
3. TASK 16 deve implementar a sessão de pairing, transcript canônico, assinaturas, nonces, código comparável, confirmação bilateral, expiração e replay protection.
4. TASK 17 deve implementar trust, revogação, fechamento/invalidação de sessões e re-pairing explícito.
5. TASK 18 deve materializar a matriz de capabilities, as direções, as decisões e o fail-closed em cada serviço.
6. TASK 20 deve ligar a identidade ao QUIC/TLS 1.3, impedir certificado arbitrário, fixar ALPN/versão e manter operações com efeito fora de 0-RTT.
7. TASKS 21 e 22 devem versionar envelopes, validar payloads, centralizar limites, aplicar anti-replay, backpressure e erros não reveladores.
8. TASK 05/09 devem expor somente intenções, estados e erros serializáveis; nunca transportar chave privada, `Secret`, socket ou detalhe de keyring para Vue.
9. TASK 06 deve criar fixtures negativas para MITM, replay, downgrade, peer revogado, capability errada, parâmetro inválido, path traversal e comando fora da allowlist.

## Execução paralela

A investigação foi paralelizada em dois recortes sem escrita sobreposta:

- **Threat model e criptografia:** revisão das fronteiras de confiança, ameaça de MITM/replay/downgrade, identidade por chave pública, pairing autenticado e armazenamento seguro, apoiada em fontes primárias.
- **Auditoria do contrato local:** cruzamento de `PRODUCT.md`, `DESIGN.md`, `SYSTEM-DESIGN.md`, TODO, TASKS 01/02 e dos modelos TypeScript/Rust para localizar estados, lacunas e conflitos entre trust e capability.

A consolidação, as decisões e a revisão deste arquivo permanecem sequenciais. Nenhum subagente editou código ou documentação.

## Integração

- A TASK 02 deve continuar tratando mDNS/DNS-SD e QUIC como mecanismos não confiáveis até que a identidade seja validada.
- A TASK 04 não pode persistir segredo em SQLite/arquivo comum só porque trust e grants precisam ser armazenados; material privado continua no keyring do SO.
- As TASKS 11–14 devem mostrar candidatos e presença sem rotulá-los como confiáveis.
- As TASKS 16–19 devem usar esta política para pairing, revogação, grants e copy, mantendo estados offline/stale separados de revoked/denied.
- As TASKS 20–22 devem garantir que uma sessão de transporte não bypassa trust, capability, validação de payload, limites ou anti-replay.
- A TASK 23 em diante deve tratar paths, Clipboard, mídia e comandos como efeitos locais sob owner/capability, nunca como texto livre vindo da rede.
- O modo mockado, o command `greet`, as rotas e as capabilities Tauri atuais permanecem inalterados nesta task.

## Critérios de conclusão

- [x] Ameaças locais, adversários, ativos, fronteiras e limites de escopo estão documentados.
- [x] Spoofing, MITM, replay, downgrade, payload malformado, vazamento de caminho, peer não pareado e execução não autorizada têm controles e resultados esperados.
- [x] A identidade está ancorada em chave pública estável, com `DeviceId`, fingerprint, rotação e perda de identidade definidos.
- [x] Pairing exige prova de posse, transcript vinculado, nonces, expiração, código comparável e confirmação explícita nos lados envolvidos.
- [x] Armazenamento de segredo, keyring bloqueado/indisponível, exposição em logs/UI e comportamento fail-closed estão definidos.
- [x] Trust, presença, capability, revogação, re-pairing e invalidação de sessões têm decisões separadas e verificáveis.
- [x] A matriz cobre todas as capabilities canônicas, direção, default, consequência e controles adicionais.
- [x] Os limites de `commands.execute` excluem shell arbitrário e exigem allowlist, parâmetros validados, aprovação e auditoria.
- [x] A documentação marca claramente que nenhuma comunicação ou segurança de produção foi implementada nesta task.

## Validação

### Evidência documental e fontes primárias

- Revisão cruzada de `PRODUCT.md:27-33,56-63`, `DESIGN.md` (regras de confirmação e estados), `SYSTEM-DESIGN.md:138-189`, `TODO.md:37-46`, TASK 01 e TASK 02.
- Revisão dos tipos e transições em `src/types/index.ts:32-55,133-179,346-375,437-525` e `src-tauri/src/domain/mod.rs:84-186,245-297,630-705`.
- [RFC 8446 — TLS 1.3](https://www.rfc-editor.org/rfc/rfc8446.html): autenticação, transcript e acordo de chaves.
- [RFC 9001 — Using TLS to Secure QUIC](https://www.rfc-editor.org/rfc/rfc9001.html): TLS 1.3 no QUIC, autenticação de peers, ALPN e cuidados com 0-RTT.
- [RFC 8032 — EdDSA: Ed25519 and Ed448](https://www.rfc-editor.org/rfc/rfc8032.html): assinatura e verificação da identidade pública.
- [RFC 7748 — Elliptic Curves for Security](https://www.rfc-editor.org/rfc/rfc7748.html) e [RFC 5869 — HKDF](https://www.rfc-editor.org/rfc/rfc5869.html): acordo/derivação quando exigidos pela implementação.
- [Secret Service API — Collection and Items](https://specifications.freedesktop.org/secret-service/latest/ch03.html): collections/items, lookup attributes e comportamento de segredo bloqueado no Linux.
- [NIST SP 800-57 Part 1 Rev. 5](https://nvlpubs.nist.gov/nistpubs/specialpublications/nist.sp.800-57pt1r5.pdf): proteção, ciclo de vida e revogação de material criptográfico.

### Revisão de cenários negativos

| Cenário | Resultado exigido |
| --- | --- |
| Dois candidatos com o mesmo nome | Permanecem distintos até identidade autenticada; nenhum ganha trust por nome. |
| MITM troca chaves no pairing | Códigos/transcripts divergem ou fingerprint não confere; sessão termina sem trust. |
| Confirmação antiga repetida | `PairingSession` de uso único rejeita replay; nenhum novo grant é criado. |
| Versão/protocolo inferior | Negociação falha antes de recurso; não há downgrade silencioso. |
| Peer offline/stale | Operação falha como indisponível; trust e grants não são revogados automaticamente. |
| Peer revogado reaparece | Sessão autorizada é negada; só re-pairing local explícito pode iniciar nova decisão. |
| `files.receive` usado como `files.send` | Verificação de direção nega a operação. |
| Link, path ou payload acima do limite | Validação falha sem abrir link, escrever arquivo ou executar efeito. |
| `commands.execute` com shell/parâmetro desconhecido | Ação fora da allowlist é rejeitada e auditada. |
| Keyring bloqueado ou ausente | Identidade estável/pairing não ficam disponíveis; nenhuma chave vai para arquivo comum. |

### Validação de implementação

Não há implementação de código nesta task, portanto não há mudança de runtime a validar. As validações `npm run typecheck`, `npm run build` e `cargo check --manifest-path src-tauri/Cargo.toml` continuam sendo responsabilidade das tasks que alterarem código; a revisão desta task deve confirmar que o working tree não recebeu dependências, commands, capabilities Tauri, sockets ou segredos.
