# TASK 04 — Decidir persistência, migrações e retenção local

Status: decisão de armazenamento concluída; implementação fica para as TASKS 07, 08, 15, 17, 18, 27, 30 e 33

## Objetivo

Definir como o Pulse armazenará estado confiável e histórico local sem depender de cloud, sem colocar segredos em arquivos comuns e sem transformar conteúdo sensível em log. Esta task é documental e de decisão: não adiciona banco, migrações, comandos Tauri, keyring, histórico funcional ou persistência aos mocks atuais.

## Estado atual

- O produto ainda é uma fundação navegável `0.1.0`; dispositivos, transferências e bridge são demonstrativos e não há persistência nem hidratação (`PRODUCT.md:5-7,38-47`, `SYSTEM-DESIGN.md:68-76`).
- Os modelos canônicos já distinguem identidade, presença, pairing, trust, capability, transferência, Clipboard, histórico e notificação, mas não definem armazenamento (`src/types/index.ts:91-173,235-335,380-525`, `src-tauri/src/domain/mod.rs:205-350,420-590,740-778`).
- `DiscoveryCandidate` contém endpoint e capacidades observadas transitórias; `Device`, `TrustRelationship` e `CapabilityGrant` contêm os metadados que poderão sobreviver a reinícios (`src/types/index.ts:98-173`).
- `TransferItem` pode conter `LocalPath` e `LightContent` pode conter texto ou URL; esses valores são conteúdo operacional e não devem ser copiados automaticamente para histórico ou banco (`src/types/index.ts:181-235`).
- O threat model exige chave privada no armazenamento seguro do sistema, fail-closed quando o keyring não estiver disponível e histórico sem segredo, conteúdo de Clipboard, caminho completo ou payload bruto (`docs/tasks/TASK-03-threat-model-identidade-trust-capabilities.md:82-88,114-122,145-149`).
- O runtime Rust possui apenas `greet`, sem serviços, conexão de banco, bridge de produto ou capability Tauri de arquivos/rede (`src-tauri/src/lib.rs`, `src-tauri/Cargo.toml`, `src-tauri/capabilities/default.json`, `SYSTEM-DESIGN.md:78-104`).

## Brainstorm

### Alternativas consideradas

| Alternativa | Avaliação | Decisão |
| --- | --- | --- |
| SQLite controlado pelo Rust | Transações, índices, consultas de histórico, integridade referencial e migrações versionadas cabem em um único arquivo local. Mantém o domínio atrás da bridge. | Escolhida |
| JSON/TOML em vários arquivos | Simples no primeiro uso, mas exige atomicidade, locking, recuperação e migração próprios; relações entre trust, grants, eventos e transferências ficam frágeis. | Rejeitada |
| `tauri-plugin-sql` exposto à UI | Tem suporte a SQLite e migrations, mas seu contrato público executa `load`, `execute` e `select` por comandos do plugin. Isso permitiria SQL genérico no frontend e atravessaria a fronteira de domínio definida pelo Pulse. | Rejeitada para o armazenamento de produto |
| Banco remoto/cloud | Contradiz local-first, adiciona dependência de rede e não resolve a necessidade de estado disponível offline. | Rejeitada |
| Banco embutido chave-valor | Pode servir a preferências simples, mas não oferece a mesma clareza para consultas, retenção, deduplicação de eventos, relações e migrações do histórico. | Rejeitada para o núcleo |

### Perguntas que a decisão precisa responder

1. Qual dado precisa sobreviver a reinício para preservar confiança e recuperar estado sem sugerir uma presença online inexistente?
2. Qual dado é sensível demais para uma base local comum, mesmo que o arquivo esteja protegido pelo usuário do sistema?
3. Como um upgrade, downgrade não suportado, falha de escrita ou corrupção falha sem apagar dados válidos silenciosamente?
4. Como remoção, reset e retenção evitam deixar cópias esquecidas de histórico, Clipboard, caminhos ou segredos?

## Decisões

### 1. Backend e fronteira de acesso

- O backend de persistência será SQLite, acessado por um adaptador Rust nativo e mantido atrás dos serviços de domínio. A escolha da crate e sua versão fica para a TASK 08, quando dependências e testes forem introduzidos.
- O arquivo principal será `pulse.sqlite` no diretório retornado por `appLocalDataDir()`/`app_local_data_dir` do Tauri, dentro do identificador `com.pulse.desktop`. O caminho não será montado manualmente a partir de `$HOME` nem ficará no diretório do executável.
- No Linux, a abstração do Tauri aponta para o diretório de dados local do usuário; a convenção XDG define esse tipo de dado como pertencente ao espaço de dados do usuário. O código deve continuar usando a API de caminho do Tauri para preservar a portabilidade futura.
- O frontend não terá acesso ao arquivo, à conexão ou a SQL arbitrário. A TASK 05/09 exporá intenções e modelos tipados; nenhum comando genérico `query`, `executeSql` ou equivalente será criado.
- O banco conterá metadados de domínio não secretos. Ele não será tratado como mecanismo de armazenamento seguro nem como substituto do Secret Service.
- A conexão deve habilitar foreign keys, usar journaling e sincronização compatíveis com durabilidade local, definir timeout de contenção e rejeitar configurações que desativem a recuperação segura (`synchronous=OFF`, `journal_mode=OFF` ou equivalente). WAL pode ser usado para separar leitores e escritores, mas seus arquivos auxiliares devem ser tratados como parte do banco durante cópia, reset e diagnóstico.

### 2. Separação entre banco, keyring e dados efêmeros

| Camada | Guarda | Não guarda |
| --- | --- | --- |
| Secret Service do sistema | Chave privada Ed25519 da identidade local e eventual material privado futuro, usando atributos estáveis | Histórico, nome, caminho, token da bridge ou conteúdo de Clipboard |
| SQLite local | Identidade pública, dispositivos conhecidos, trust, grants, bloqueios, preferências e metadados de operações | Chave privada, nonce, código de pairing, assinatura/transcript, token de sessão ou segredo de transporte |
| Memória/runtime | Presença atual, candidatos, endpoints, sessões QUIC, nonces, conteúdo em trânsito e estado transitório da UI | Qualquer garantia de sobrevivência a reinício |
| Diretórios de trabalho dos recursos | Arquivos parciais e staging de transferências, sob a política de arquivos das tasks próprias | Conteúdo como coluna genérica do banco ou entrada de histórico |

Se o Secret Service estiver ausente, bloqueado ou inconsistente, a identidade local não será criada ou carregada de forma estável. O banco pode abrir para diagnóstico e preferências não sensíveis, mas pairing, trust operacional e recursos autorizados devem permanecer indisponíveis até a identidade ser recuperada por uma ação explícita.

### 3. Esquema lógico inicial

A TASK 08 transformará este mapa em migrations SQL e modelos de repositório. Os nomes são contratos de armazenamento, não novos tipos públicos da bridge.

| Tabela/registro | Persistir | Não persistir | Retenção/remoção |
| --- | --- | --- | --- |
| `schema_migrations` | Versão, nome, checksum, momento e resultado da migration aplicada | SQL executado, segredo ou payload | Histórico técnico necessário para compatibilidade; nunca editar uma migration já aplicada |
| `local_identity_public` | Formato, algoritmo, chave pública, `DeviceId`, fingerprint e timestamps | Chave privada, seed, segredo de recuperação ou senha | Até reset explícito; rotação cria novo registro/identidade conforme TASK 15 |
| `known_devices` | `DeviceId`, chave pública/fingerprint autenticados, nome/metadados apresentados, plataforma e `last_seen_at` | Endpoint/IP/porta transitórios, capacidades anunciadas stale e presença `online` | Até “esquecer dispositivo” ou reset; `last_seen_at` não restaura online |
| `trust_relationships` | Estado `unpaired`/`trusted`/`revoked`, pairing de origem, decisão, revogação e motivo | Transcript, nonces, código curto e prova assinada | Até nova decisão explícita, remoção autorizada ou reset; revogação ativa não some por expiração de presença |
| `capability_grants` | Chave, direção, estado, origem, timestamps e motivo por dispositivo | Capability temporária de anúncio sem decisão, token ou mensagem bruta | Até revogação/remoção/reset; trust revogado revoga grants em transação |
| `revocation_blocks` | Identidade/fingerprint bloqueado, epoch/momento, origem e motivo | Chave privada ou endpoint | Enquanto o bloqueio estiver ativo; só uma ação local explícita de re-pairing pode encerrá-lo |
| `pairing_sessions` | IDs, identidade pública apresentada, estado terminal, expiração, timestamps e código de erro não sensível | Nonces, short code, transcript, assinaturas, endpoint e payload | Sessões ativas expiram em 2 minutos; após reinício são encerradas como `expired`/`failed`; resumo terminal segue retenção de histórico |
| `transfer_sessions` | IDs, origem/destino, tipo, estado, tentativa, contagens/tamanhos, timestamps, erro e integridade | Conteúdo, URL/texto, caminho completo, token de retomada ou payload de protocolo | Estados ativos são recuperáveis como interrompidos; metadados terminais por 30 dias ou limite de volume definido na TASK 27 |
| `history_entries` | Tipo, origem, destino, capability/direção quando aplicável, resultado, IDs relacionados, timestamps e motivo | Conteúdo de Clipboard, texto/link, caminho completo, IP/porta, chave, token e payload bruto | 90 dias ou 10.000 entradas, o que ocorrer primeiro; limpeza manual pode remover entradas não necessárias para bloqueios de segurança |
| `notification_records` | ID do evento, severidade, chaves de copy, estado e expiração | Conteúdo livre recebido, segredo ou payload remoto | Até dispensada/expirada; máximo de 7 dias |
| `preferences` | Preferências locais não secretas, incluindo retenção, notificações e UX | Credenciais, chave privada, conteúdo ou path arbitrário | Até alteração ou reset |

`last_seen_at` é uma observação histórica mínima, não um estado de presença. Na hidratação, a presença começa como `unknown`/`offline` conforme o serviço, e só passa a `online` depois de uma observação válida. Candidatos, endpoints, capabilities `available` anunciadas, mídia e presença corrente não são fonte persistente de confiança.

### 4. Clipboard, conteúdo leve, caminhos e logs

- O payload de Clipboard nunca será persistido na política padrão da v1, nem em SQLite, histórico, logs, notificações ou fixtures. A UI pode exibir o valor enquanto ele estiver em memória e autorizado.
- Texto e links enviados como `LightContent` não serão copiados para histórico; o registro guarda apenas tipo, tamanho, resultado, origem/destino e motivo. URL não é metadado neutro.
- O histórico não guarda caminho completo. Transferências podem precisar de metadados operacionais e um mecanismo futuro de retomada, mas a implementação não persistirá `LocalPath` cru para tornar o reinício automático conveniente.
- Nome de arquivo pode ser armazenado somente no estado operacional da transferência quando necessário para apresentação/manifesto; nunca será concatenado em logs ou usado como caminho de destino sem a validação das TASKS 23–27.
- Logs técnicos devem ser separados do histórico de produto, redigidos e limitados. Erros de keyring, bridge e transporte não podem imprimir chave, token, conteúdo, path completo, URL ou payload.

### 5. Migrations, versões e compatibilidade

- O schema terá versão própria, distinta de `DOMAIN_MODEL_VERSION` e da versão do aplicativo. A tabela `schema_migrations` será a fonte operacional; `PRAGMA user_version` pode servir como marcador de diagnóstico, não como substituto do checksum.
- Migrations são somente para frente em builds publicados, numeradas, versionadas no repositório, com descrição e checksum. Uma migration aplicada não pode ser editada; correções recebem nova versão.
- Cada migration roda dentro de uma transação atômica, com foreign keys e invariantes verificadas. A inicialização não expõe serviços de produto até o schema estar na versão suportada.
- Instalação nova cria o diretório/banco e aplica a sequência completa. Uma versão anterior aplica apenas migrations posteriores, preservando dados válidos.
- Se o banco tiver versão maior que a suportada pelo binário, o app falha com estado explícito `storage-incompatible` e não tenta downgrade, apagar tabelas ou abrir o banco em modo de escrita.
- Se a migration falhar, a transação é revertida; o banco original permanece intacto e os serviços dependentes não iniciam. A tarefa de runtime deverá expor erro recuperável e diagnóstico sem vazar SQL, path ou dados.
- Antes de uma migration destrutiva futura, a TASK 08 deverá criar uma cópia de recuperação local consistente ou usar o mecanismo de backup do SQLite com permissão restrita. Essa cópia obedece à mesma retenção e será removida junto com o reset; não haverá backup cloud automático.

### 6. Corrupção, falha de escrita e recuperação

- Na abertura, o storage valida header/identificador do banco, versão, foreign keys e integridade suficiente para o modo de inicialização. Em diagnóstico/manutenção, `quick_check`/`integrity_check` e `foreign_key_check` devem distinguir corrupção estrutural de erro de aplicação.
- Corrupção, arquivo ilegível, lock persistente, filesystem sem espaço ou falha de sincronização não viram banco vazio silenciosamente. O app preserva o arquivo original, impede operações de trust/transferência e mostra recuperação necessária.
- A recuperação automática não fará “best effort” destrutivo. O banco com problema será isolado/quarentenado com seus arquivos auxiliares quando possível; qualquer salvamento parcial exige ação explícita e ferramenta/teste dedicado.
- Falhas de escrita durante uma transação não devem deixar metade de uma decisão: revogar trust e grants, registrar evento e fechar sessões será uma única unidade transacional quando pertencer ao mesmo serviço.
- Ausência de espaço ou permissão negada é erro recuperável, não motivo para remover histórico ou recriar identidade. A UI poderá oferecer tentar novamente, liberar espaço ou resetar dados explicitamente.

### 7. Reset, remoção e downgrade

- “Limpar histórico” remove apenas entradas elegíveis de `history_entries` e notificações expiradas; não apaga trust, grants, identidade ou bloqueios ativos.
- “Esquecer dispositivo” remove metadados e grants conforme a política da TASK 17, mas não deve remover silenciosamente um bloqueio de segurança. Desbloqueio/re-pairing será uma ação separada e auditável.
- “Resetar dados locais” exige confirmação explícita, encerra serviços, fecha o banco, remove dados de domínio, staging e backups locais e solicita a remoção da identidade do Secret Service. O processo só pode criar nova identidade depois de confirmar armazenamento seguro.
- Não há downgrade automático. Remover o app não é tratado como recuperação; o procedimento de desinstalação e suporte deverá documentar que dados locais e segredo do keyring podem exigir limpeza separada.
- Um reset não apaga evidência externa já criada pelo peer nem promete revogar uma cópia de conteúdo que já tenha sido recebida; ele apenas remove o estado local sob controle do Pulse.

## Decisões rejeitadas

| Decisão rejeitada | Motivo |
| --- | --- |
| Persistir a chave privada junto do SQLite | Viola a política da TASK 03 e transforma uma cópia comum em autoridade da identidade. |
| Persistir Clipboard/texto/links para “melhorar histórico” | Aumenta exposição de conteúdo e URL sem necessidade para a leitura de resultado. |
| Marcar presença como `online` após hidratação | Confunde `lastSeenAt` com liveness e pode autorizar UX/ações indevidas. |
| Apagar e recriar banco quando migration falhar | Perde trust, grants e histórico silenciosamente e pode gerar identidade inconsistente. |
| Downgrade automático ou migrations `down` em release | Pode destruir dados e não oferece uma semântica segura para versões antigas. |
| SQL genérico na bridge/frontend | Acopla UI a schema interno, amplia superfície de comando e permite bypass das regras de domínio. |

## Plano de implementação

Esta task não implementa o storage. As próximas tarefas devem seguir esta ordem:

1. TASK 07 cria o serviço de storage como dependência explícita do runtime, com estado de inicialização parcial e erro recuperável.
2. TASK 08 adiciona a crate SQLite escolhida, abre `appLocalDataDir`, cria migrations versionadas, aplica pragmas, repositórios Rust e testes de instalação/migração/falha.
3. TASK 15 integra a identidade pública ao banco e a chave privada ao Secret Service, sem transportar segredo para a bridge.
4. TASK 17/18 usam transações para trust, revogação, bloqueios e grants; a presença continua sendo runtime.
5. TASK 24/27 persistem somente metadados de sessões e definem retomada/interrupção sem path cru ou payload no banco.
6. TASK 30 implementa histórico append-only lógico, retenção, paginação, deduplicação e remoção explícita.
7. TASK 33 persiste preferências e estado de notificação sem perder eventos de histórico.

## Execução paralela

A investigação foi dividida em recortes independentes, sem escrita sobreposta:

- **Contrato e segurança:** cruzamento dos modelos TypeScript/Rust, da política de identidade/trust/capabilities e dos requisitos de histórico, Clipboard, paths e reset.
- **Persistência e plataforma:** comparação de SQLite, arquivos soltos, plugin SQL, diretórios do Tauri/XDG, transações, journaling, migrations e recuperação.

A consolidação desta decisão foi sequencial. Não houve implementação paralela: o schema e a fronteira do storage precisam permanecer coerentes antes de adicionar dependências ou comandos.

## Integração

- A TASK 03 continua sendo a fonte da regra de que material privado fica no keyring; esta task define apenas a fronteira do banco para metadados não secretos.
- A TASK 05/09 deve expor estados de storage como `loading`, `ready`, `storage-incompatible`, `corrupt`, `read-only` ou erro equivalente sem expor SQL ou detalhes sensíveis.
- A TASK 08 deve manter os mocks e o command `greet` inalterados; a existência deste plano não significa que a UI atual esteja persistida.
- A TASK 13 pode usar `last_seen_at` do registro conhecido, mas deve reidratar presença com estado honesto e resolver endpoint novamente por discovery.
- A TASK 30 deve obedecer à retenção e à exclusão definidas aqui; histórico de segurança não pode ser usado como depósito de payload.
- A TASK 51 deverá transformar as regras de compatibilidade, reset, remoção e recuperação em procedimento de release.

## Critérios de conclusão

- [x] O backend escolhido é SQLite local, com acesso exclusivo do Rust e sem dependência de cloud.
- [x] A localização do banco, a separação em relação ao Secret Service e a ausência de acesso SQL cru na UI estão definidas.
- [x] O esquema inicial cobre identidade pública, dispositivos, trust, grants, bloqueios, pairing, transferências, histórico, notificações e preferências.
- [x] O documento diferencia estado durável, estado transitório, payload sensível e staging de recursos.
- [x] Migrations têm versão própria, checksum, execução transacional, política de forward-only e comportamento explícito para versão futura.
- [x] Primeiro uso, upgrade, downgrade não suportado, falha de escrita, corrupção, falta de espaço, reset e remoção têm resultados definidos.
- [x] Clipboard, texto, links, paths, tokens, nonces, transcripts, chaves privadas e payloads brutos têm política explícita de não retenção.
- [x] Retenção de trust/grants, sessões, histórico, notificações e preferências está documentada com remoção e limites.
- [x] A decisão está marcada como planejada e não é apresentada como persistência implementada no app atual.

## Validação

### Evidência local

- `TODO.md:55-60,85-90,249-253,409-411` — objetivo, dependências e validação da TASK 04, TASK 08 e histórico.
- `SYSTEM-DESIGN.md:68-76,181-189` — stores efêmeros, ausência de hidratação e regra de histórico sem conteúdo sensível.
- `src/types/index.ts:98-173,181-235,285-335,380-525` e `src-tauri/src/domain/mod.rs:205-350,420-590,740-778` — entidades, IDs, timestamps, paths, conteúdo leve, transferências e eventos.
- `docs/tasks/TASK-03-threat-model-identidade-trust-capabilities.md:82-88,114-122,145-149,177-208` — keyring, fail-closed, revogação, auditoria sem payload e dependências da TASK 04.

### Fontes primárias consultadas

- [SQLite — Atomic Commit](https://www.sqlite.org/atomiccommit.html): atomicidade de transações, journaling e recuperação após crash/power loss.
- [SQLite — File Format](https://www.sqlite.org/fileformat.html): `user_version`, `application_id` e arquivos auxiliares do formato.
- [SQLite — PRAGMA](https://sqlite.org/pragma.html): modos de journal e verificações `integrity_check`, `quick_check` e foreign keys.
- [Tauri 2 — Path API](https://v2.tauri.app/reference/javascript/api/namespacepath/): `appDataDir`, `appLocalDataDir`, `dataDir` e diretórios de dados por aplicação.
- [XDG Base Directory Specification](https://specifications.freedesktop.org/basedir/): convenções de dados, estado, cache e runtime por usuário.
- [Tauri SQL plugin](https://docs.rs/crate/tauri-plugin-sql/latest): migrations e comandos SQL disponíveis, usados para rejeitar a exposição de SQL genérico à UI neste desenho.

### Cenários de validação futura

| Cenário | Resultado exigido |
| --- | --- |
| Primeira execução sem banco | Diretório é criado, schema completo é aplicado e nenhum segredo é gravado em arquivo comum. |
| Upgrade com dados válidos | Migrations posteriores rodam em ordem, preservam registros e atualizam a versão uma vez. |
| Migration interrompida | Transação reverte; o banco anterior permanece abrível e nenhum serviço inicia parcialmente. |
| Binário mais antigo abre banco novo | Inicialização falha como incompatível; não há downgrade, drop ou reset automático. |
| Banco corrompido | App preserva/quarentena o original, não cria estado vazio e oferece recuperação explícita. |
| Keyring bloqueado | UI mostra indisponibilidade de identidade; pairing e recursos autorizados ficam bloqueados. |
| Revogação com grants | Trust, grants e bloqueio mudam atomicamente e o histórico registra somente metadados não sensíveis. |
| Reinício durante transferência | Sessão não fica `completed` por inferência; torna-se interrompida/falha recuperável sem salvar payload ou path cru. |
| Retenção vencida | Job de limpeza remove apenas dados elegíveis, sem apagar trust/bloqueios ativos ou preferências. |
| Reset explícito | Serviços encerram, banco/staging são removidos e a identidade do keyring é tratada separadamente com confirmação. |

### Validação desta task

Não há alteração de runtime, dependência ou schema nesta task. Portanto, não há `npm run typecheck`, `npm run build` ou `cargo check` adicional a executar por causa do código; a validação é documental e deve confirmar que os mocks, o command `greet`, as capabilities atuais e o estado de “sem persistência” continuam inalterados.
