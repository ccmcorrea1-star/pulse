# TASK 01 — Fechar os modelos de domínio e seus estados

Status: implementação concluída; integração de estado e bridge ainda não iniciadas

## Objetivo

Definir o vocabulário canônico do Pulse para que Rust, bridge e Vue representem os mesmos recursos, estados, transições e resultados. O contrato deve ser independente de transporte, persistência e componentes visuais.

Esta task não transforma os mocks atuais em integração real e não conecta discovery, pairing ou recursos locais.

## Estado atual

- Antes da implementação, `src/types/index.ts:1-23` continha apenas `Device`, `Transfer`, `TransferStatus` e `BridgeState`; `Device` misturava identidade e presença em `online`/`lastSeen`, enquanto `Transfer` misturava estado e dados de apresentação. Esses registros agora estão nomeados como `MockDevice` e `MockTransfer`.
- `src/stores/devices.ts:6-34` e `src/stores/transfers.ts:6-31` são stores efêmeros com fixtures fixas, textos relativos como `"agora"` e progresso demonstrativo.
- `src/components/device/DeviceList.vue:19-30` e `src/components/transfer/TransferPreview.vue:19-29` inferem estado visual diretamente de `online`, `progress` e status do mock.
- `src/views/HomeView.vue:20-24,77-92` identifica explicitamente a UI como fundação/mock; `src/views/TransfersView.vue:10-22` e `src/views/DeviceSectionView.vue:16-23` mantêm placeholders honestos.
- `src-tauri/src/lib.rs` continua registrando somente `greet` e agora expõe o módulo puro `domain`; ainda não há commands de produto, estado gerenciado ou eventos Tauri.
- `src/composables/useRustBridge.ts:3-20` só diferencia prévia web e chamada `greet`; `src-tauri/tauri.conf.json:13-31` mantém `withGlobalTauri: false`, CSP nula e bundle desativado.
- `src-tauri/capabilities/default.json:3-6` concede somente `core:default`; isso é capability Tauri e não deve ser confundido com capability de autorização entre peers.

Os tipos canônicos definidos nesta task não devem ser acoplados aos nomes ou ao formato desses mocks. A adaptação dos stores fica para a TASK 10.

## Brainstorm

### Arquitetura e domínio

- `DiscoveryCandidate`, `Device`, `Presence`, `PairingSession`, `TrustRelationship`, `CapabilityGrant`, `TransferSession`, `LightContent`, `HistoryEntry`, `LocalNotification`, `MediaState`, `RemoteCommand` e `DomainEvent` devem ser entidades conceitualmente separadas.
- `CandidateId` é efêmero; `DeviceId` é opaco e estável após autenticação. Nome, IP, plataforma e `lastSeenAt` não são identidade.
- O ciclo do anúncio deve ficar no candidato (`discovered` → `expired`); a presença do dispositivo deve ficar em `unknown`, `online`, `stale` ou `offline`.
- `Presence` não altera `TrustRelationship`; capability disponível não é capability concedida; notificação local não é resultado remoto.
- Alternativa rejeitada: expandir `Device`/`Transfer` atuais. Isso preservaria ambiguidades como `online`, `lastSeen`, `deviceName` e percentual sem unidade em vez de criar um contrato canônico.

Evidências: `src/types/index.ts:3-21`, `src/stores/devices.ts:6-34`, `src/stores/transfers.ts:6-31`, `PRODUCT.md:27-33` e `SYSTEM-DESIGN.md:136-159`.

### Frontend e Vue

- A UI atual já exibe online/offline, percentual, fila e estados de bridge, mas sempre com origem demonstrativa; isso deve continuar até a TASK 10.
- O modelo canônico não deve carregar copy localizado, tempo relativo, cor, ícone ou label pronto para tela. Um adaptador de apresentação deve derivar esses valores.
- `lastSeen: "agora"`/`"há 18 min"` deve virar timestamp no domínio e formatter na apresentação. Candidatos não devem ser adaptados silenciosamente ao mesmo view model de dispositivos confiáveis.
- A barra de progresso atual separa o percentual do elemento visual; a futura representação deve fornecer unidade e texto acessível, não apenas um número ou cor.

Evidências: `src/components/app/Sidebar.vue:49-72`, `src/components/device/DeviceList.vue:15-30`, `src/components/transfer/TransferPreview.vue:19-29`, `src/views/DeviceView.vue:23-35` e `DESIGN.md:39-48,99-119`.

### Rust, Tauri e bridge

- A bridge atual é um smoke test: `greet` recebe `&str`, retorna `String` e não modela erro, estado ou eventos (`src-tauri/src/lib.rs:3-13`).
- Os modelos canônicos devem ser puros e independentes de `tauri::State`, `AppHandle`, `invoke`, listeners, nomes de commands ou envelope IPC.
- A bridge futura precisará transportar IDs, timestamps, enums e erros serializáveis; o formato wire, os commands e os eventos pertencem à TASK 05.
- Capability do Pulse é autorização entre dispositivos; capability Tauri é permissão de janela/webview. Não são o mesmo contrato.

Referências arquiteturais: `SYSTEM-DESIGN.md:78-93,136-148`. Referências oficiais consultadas para o limite futuro: [Calling Rust from the Frontend](https://v2.tauri.app/develop/calling-rust/), [Calling the Frontend from Rust](https://v2.tauri.app/develop/calling-frontend/) e [Tauri Capabilities](https://v2.tauri.app/security/capabilities/).

### Segurança e testes

- O contrato precisa impedir que presença represente confiança, suporte represente autorização ou notificação represente sucesso.
- Cenários negativos prioritários incluem peer revogado que reaparece, pairing expirado que vira trust, capability usada na direção errada, transferência concluída sem integridade, progresso inválido, retry duplicado e comando fora da lista permitida.
- Na execução da TASK 01, não existia runner, script de teste, fixture canônica ou harness Rust/Vue (`package.json:6-34`, `src-tauri/Cargo.toml:15-16`). A TASK 06 criou essa infraestrutura e exercita os invariantes definidos aqui.

## Decisões

- **Modelo canônico separado dos mocks:** definir entidades e estados sem renomear ou adaptar os stores atuais nesta task. A migração de stores/componentes fica para a TASK 10.
- **Identidade e tempo:** usar IDs opacos por entidade e timestamps absolutos em UTC, preferencialmente RFC 3339; duração de heartbeat e timeout não será representada como texto.
- **Candidato versus presença:** `DiscoveryCandidate` terá ciclo próprio de anúncio (`discovered`, `expired`). `Presence` usará `unknown`, `online`, `stale` e `offline`. `revoked` pertence a trust, nunca à presença.
- **Pairing versus trust:** `PairingSession` será temporária e terá `requested`, `awaiting-confirmation`, `confirmed`, `rejected`, `expired`, `canceled` e `failed`; `TrustRelationship` terá `unpaired`, `trusted` e `revoked`. Reaparecer online não reativa trust.
- **Capability em dois eixos:** suporte anunciado (`available`) será distinguido da decisão de autorização (`requested`, `granted`, `denied`, `revoked`). Uma capability desconhecida ou não concedida deve falhar fechada.
- **Transferência unificada:** arquivos, pastas, texto e links usarão a mesma noção de `TransferSession`, com estados `draft`, `awaiting-approval`, `queued`, `active`, `paused`, `completed`, `failed` e `canceled`. Progresso sempre declarará sua base.
- **Efeitos separados de resultados:** `HistoryEntry` registra o resultado da operação; `LocalNotification` registra apenas a entrega local do aviso. Mídia observada não autoriza controle; `RemoteCommand` não representará shell arbitrário.
- **Bridge adiada:** esta task fecha semântica e invariantes, não nomes de commands, envelope JSON, tipo de erro, mecanismo de eventos ou estratégia de streaming. Esses pontos dependem da TASK 05 e das restrições do transporte.
- **Validação adiada, invariantes não:** não escolher uma biblioteca de testes nesta task; registrar cenários e contratos para a TASK 06, sem alegar que tipos TypeScript substituem validação de runtime no Rust.

## Plano de implementação

1. Consolidar o glossário, IDs, timestamps e a separação entre candidato, dispositivo, presença, pairing, trust e capability.
2. Registrar tabelas de estados, transições permitidas, estados terminais e invariantes para cada entidade do escopo.
3. Definir o contrato semântico equivalente para TypeScript e Rust, mantendo os modelos livres de dependências de Vue, Pinia, Tauri e transporte.
4. Definir a taxonomia mínima de eventos de domínio sem fechar ainda o envelope da bridge, a entrega ou a persistência.
5. Criar um mapa de adaptação dos tipos mockados atuais para o contrato canônico, sem alterar seus dados, estado ou comportamento visual nesta etapa.
6. Revisar o contrato contra os cenários negativos de segurança e os estados textuais exigidos pelo design.
7. Registrar decisões que permanecem bloqueadas pelas TASK 02–06 e implementar os tipos somente depois de o contrato estar consolidado.

## Fontes e limites

- Produto e ordem funcional: `PRODUCT.md`.
- Entrada rápida, comandos e estado geral do repositório: `README.md`.
- Camadas e responsabilidades técnicas: `SYSTEM-DESIGN.md`.
- Estados textuais, estados vazios e comportamento acessível: `DESIGN.md`.
- Convenções de implementação e validação: `AGENTS.md`.

Ficam fora desta task a escolha do transporte, o formato de persistência, o contrato final da bridge, a criptografia, a implementação de serviços e a definição de permissões Tauri. Esses assuntos dependem deste contrato e pertencem às tasks seguintes.

## Resultado esperado

Produzir um contrato de domínio revisado e versionável, com:

- IDs opacos e relações explícitas entre entidades;
- timestamps em formato comum e significado definido para cada relógio;
- estados separados para descoberta, presença, pairing, trust e capability;
- estados observáveis para transferências, conteúdo leve, histórico, notificações, mídia e comandos remotos;
- transições válidas, estados terminais e invariantes de segurança;
- vocabulário textual que a UI possa exibir sem inferir estado a partir de cor ou de um booleano;
- distinção documentada entre modelo canônico e dados de apresentação/mock.

## Modelo canônico a fechar

### Identidade e tempo

- `DeviceId`: identificador opaco e estável da identidade autenticada de um dispositivo. Nome, plataforma, endereço de rede e `lastSeen` não substituem esse ID.
- `CandidateId`: identificador efêmero de um anúncio ainda não autenticado. Não deve ser tratado como dispositivo confiável.
- IDs de pairing, transferência, evento de histórico, notificação e comando remoto devem ser opacos, únicos no escopo do app e independentes de índice de array.
- Timestamps externos devem usar um formato absoluto e inequívoco em UTC, preferencialmente RFC 3339. O contrato deve distinguir `occurredAt`, `updatedAt`, `expiresAt` e `lastSeenAt`.
- Timeouts e duração de heartbeat devem ser tratados como duração, não como timestamp. Comparações de expiração não devem depender de texto como `agora` ou `há 18 min`.

### Dispositivo, candidato e presença

O contrato deve separar três conceitos:

- `DiscoveryCandidate`: anúncio transitório, com `CandidateId`, nome apresentado, plataforma declarada, endereço transitório, capacidades anunciadas e validade do anúncio.
- `Device`: identidade conhecida, metadados não sensíveis, `DeviceId`, plataforma, relação de trust e resumo das capabilities.
- `Presence`: estado observado de disponibilidade, independente de trust.

O ciclo do candidato usa `discovered` e `expired`. Estados de presença a fechar: `unknown`, `online`, `stale` e `offline`. `revoked` não é estado de presença; é estado de trust.

Invariantes:

- Um candidato descoberto nunca é automaticamente confiável.
- `offline` não significa `revoked` e `stale` não significa `offline` sem que o timeout definido seja atingido.
- `lastSeenAt` registra a última observação válida; não é prova de que o dispositivo esteja online agora.
- Mudança de endereço ou nome não cria uma nova identidade quando o `DeviceId` autenticado permanece o mesmo.

### Pairing e trust

`PairingSession` representa uma tentativa explícita e temporária de estabelecer confiança. Deve conter origem, destino, identidade apresentada, momento de criação, expiração e resultado.

Estados de pairing a fechar: `requested`, `awaiting-confirmation`, `confirmed`, `rejected`, `expired`, `canceled` e `failed`.

`TrustRelationship` representa a relação persistida entre este Pulse e um `DeviceId`. A presença do peer não altera essa relação.

Estados de trust a fechar: `unpaired`, `trusted` e `revoked`.

Invariantes:

- Somente uma confirmação explícita e verificável pode criar `trusted`.
- `rejected`, `expired` e `canceled` encerram a sessão sem criar trust.
- Um dispositivo offline continua confiável; um dispositivo revogado não volta a ser confiável por reaparecer na rede.
- Novo pairing após revogação deve criar uma nova decisão explícita; não pode reativar trust silenciosamente.

### Capabilities

`CapabilityInfo` representa o suporte anunciado e `CapabilityGrant` representa a decisão de autorização de um recurso para um dispositivo específico. O modelo deve manter os dois eixos separados de pairing e identificar, quando aplicável, a direção da operação.

O conjunto inicial de chaves deve cobrir:

- `files.send` e `files.receive`;
- `clipboard.read` e `clipboard.write`;
- `text.send` e `links.send`;
- `media.read` e `media.control`;
- `notifications.receive`;
- `commands.execute`.

O eixo de suporte usa `available`; o eixo de autorização usa `requested`, `granted`, `denied` e `revoked`. Uma visão combinada pode expor esses estados, mas não deve perder a distinção semântica.

Invariantes:

- `available` significa que o peer declara suporte; não significa autorização.
- Uma operação exige trust válido e capability `granted` para o dispositivo e a direção corretos.
- `denied` e `revoked` não podem ser tratados como equivalentes a ausência de dados ou peer offline.
- Toda decisão deve ter origem, momento e motivo suficientes para ser mostrada e auditada, sem expor segredo à UI.

### Transferência e conteúdo leve

`TransferSession` deve representar tanto arquivos/pastas quanto conteúdo leve, com origem, destino, itens, tamanho conhecido quando disponível, progresso, tentativa, timestamps, erro e resultado.

Estados de transferência a fechar: `draft`, `awaiting-approval`, `queued`, `active`, `paused`, `completed`, `failed` e `canceled`.

Invariantes:

- `completed` só pode ocorrer após confirmação de integridade e conclusão do lado responsável pelo destino.
- `paused`, `failed` e `canceled` são resultados diferentes e devem permanecer distinguíveis na UI.
- Retry e retomada não podem transformar uma tentativa parcialmente gravada em conclusão nem duplicar conteúdo.
- Progresso deve indicar sua base (`bytes`, itens ou indeterminado); `64%` mockado não é progresso de uma sessão real.
- O caminho local, a política de destino e o conflito de nomes são dados controlados do domínio, não texto livre enviado pela UI.

`LightContent` deve começar com os tipos `text` e `link`, limites explícitos e payload separado de metadados de apresentação. O envio usa a mesma sessão de transferência e autorização, sem criar um canal privilegiado.

`ClipboardState` representa uma observação local ou remota de `LightContent`, identifica a origem e não implica sincronização contínua nem autorização de escrita. O conteúdo pode estar ausente quando a política local não permitir carregá-lo no estado observado.

### Histórico e notificações

`HistoryEntry` deve ser um registro local de evento relevante, com ID, tipo, origem, destino quando aplicável, resultado, timestamps e referência à entidade relacionada. Não deve ser um dump de logs técnicos nem armazenar conteúdo sensível por padrão.

Resultados de histórico a fechar: `succeeded`, `failed`, `denied`, `canceled` e `expired`.

`LocalNotification` deve representar um efeito local derivado de um evento, não uma confirmação de entrega remota. Deve conter severidade, título/copy, referência ao evento, timestamp e estado de apresentação.

Estados de notificação a fechar: `queued`, `delivered`, `dismissed`, `expired` e `failed`.

Invariantes:

- Notificação entregue localmente não prova que uma operação remota terminou com sucesso.
- Histórico de trust e resultado de operações deve sobreviver à decisão de exibir ou dispensar uma notificação.
- Retenção de Clipboard e payloads sensíveis não é definida por este modelo; deve respeitar a decisão da TASK 04.

### Mídia e comando remoto

`MediaState` deve separar disponibilidade do estado de reprodução. Estados de reprodução a fechar: `unknown`, `playing`, `paused` e `stopped`.

`RemoteCommand` deve identificar dispositivo, capability, ação, parâmetros permitidos, solicitação, resultado e timestamps. Estados a fechar: `requested`, `awaiting-approval`, `running`, `succeeded`, `rejected`, `failed`, `canceled` e `expired`.

Invariantes:

- Estado de mídia observado não autoriza controle.
- Um comando remoto exige capability específica, parâmetros validados e, quando aplicável, confirmação explícita.
- `succeeded` significa resultado confirmado pelo peer ou pelo adaptador local; aceitação da solicitação não basta.
- O modelo não deve permitir representar execução arbitrária de shell como uma ação válida.

## Eventos e representação visual

Definir uma taxonomia mínima de eventos de domínio, sem fechar ainda o envelope da bridge. Cada evento deve ter ID, tipo, entidade relacionada, origem, momento e versão do modelo.

Os nomes de evento devem permitir que a UI diferencie pelo menos:

- candidato descoberto, presença atualizada e candidato expirado;
- pairing solicitado, confirmado, recusado, expirado ou cancelado;
- trust concedido ou revogado;
- capability solicitada, concedida, negada ou revogada;
- transferência enfileirada, iniciada, pausada, retomada, concluída, falha ou cancelada;
- resultado de Clipboard/conteúdo leve, mídia e comando remoto;
- registro histórico criado e notificação local atualizada.

O contrato deve associar cada estado a um rótulo textual em português brasileiro e a uma orientação de ação/feedback. A UI não deve deduzir estados de `online`, `progress`, cor ou presença de uma mensagem opcional.

## Arquivos envolvidos

### Nesta task

- `docs/tasks/TASK-01-modelos-de-dominio-e-estados.md`: registrar o contrato, decisões, invariantes e validação desta task.
- `src/types/index.ts`: implementar os tipos canônicos TypeScript, estados, limites de conteúdo, eventos e tabelas de transição.
- `src-tauri/src/domain/mod.rs`: implementar os modelos puros Rust equivalentes e as transições sem acoplamento à bridge.

### Fora desta implementação

- `src/stores/`: o estado Vue continua mockado; a adaptação para entidades canônicas fica para a TASK 10.
- `src-tauri/src/lib.rs`: apenas registra o módulo puro de domínio; commands, eventos IPC e listeners ficam para as TASKS 05 e 09.
- Serialização, persistência, discovery, pairing real, rede e capabilities Tauri permanecem fora desta task.

## Execução paralela

A investigação foi paralelizada porque havia recortes independentes e nenhum subagente editou arquivos:

- **Subagente A — arquitetura e domínio:** `PRODUCT.md`, `SYSTEM-DESIGN.md`, `TODO.md`, esta task, `src/types/index.ts` e stores; separou entidades, estados e alternativas.
- **Subagente B — frontend/Vue:** `DESIGN.md`, views e componentes de dispositivos, transferências e bridge; levantou estados visíveis, riscos de compatibilidade e boundary de apresentação.
- **Subagente C — Rust/Tauri:** `src-tauri/src/lib.rs`, `main.rs`, `Cargo.toml`, `tauri.conf.json`, capabilities e `useRustBridge.ts`; definiu o limite entre domínio e bridge e consultou a documentação oficial do Tauri.
- **Subagente D — segurança/testes:** `PRODUCT.md`, `SYSTEM-DESIGN.md`, `AGENTS.md`, `package.json`, `Cargo.toml` e estrutura de testes; levantou invariantes, cenários negativos e lacunas de infraestrutura.

Não há paralelismo de implementação nesta etapa: os tipos TypeScript e Rust só devem ser criados depois de o vocabulário comum ser consolidado, e `src/types/index.ts` seria um arquivo compartilhado. A execução paralela futura deve usar arquivos disjuntos.

Nesta execução, a implementação foi integrada sequencialmente para revisar a equivalência dos arquivos TypeScript e Rust antes da validação. Os fixtures receberam apenas tipos explícitos de mock (`MockDevice` e `MockTransfer`); seus dados e comportamento não foram alterados.

## Integração

- Primeiro integrar as decisões semânticas deste arquivo; depois revisar os nomes e estados em TypeScript e Rust lado a lado.
- Tipos puros TypeScript e tipos puros Rust podem ser trabalhados em paralelo em arquivos disjuntos, desde que ambos sigam este contrato. Nesta execução, a equivalência foi revisada antes de qualquer bridge.
- O mapa de adaptação para `src/stores/devices.ts` e `src/stores/transfers.ts` fica sequencial e reservado à TASK 10.
- `src/types/index.ts`, `docs/tasks/TASK-01-modelos-de-dominio-e-estados.md` e o ponto de registro de módulos Rust não devem ser editados por subagentes em paralelo sem uma divisão explícita de ownership.
- `src-tauri/src/lib.rs` só recebe o registro do módulo puro; `src/composables/useRustBridge.ts` não entra na implementação. Commands, eventos, serialização e listeners ficam para as TASKS 05 e 09.
- A integração não deve alterar os mocks, rotas ou placeholders atuais nem transformar o command `greet` em API de produto.

## Critérios de conclusão

- [x] Cada entidade do escopo possui identidade, relações, timestamps e estado definidos.
- [x] Presença, pairing, trust e capability são independentes e não podem ser confundidos por um booleano.
- [x] As transições válidas, terminais e invariantes de cada ciclo foram revisadas.
- [x] O contrato contempla arquivos/pastas, texto/links, Clipboard, histórico, notificações, mídia e comandos remotos sem prometer comportamento ainda não implementado.
- [x] O vocabulário textual da UI representa loading, stale, offline, erro, recusa, cancelamento e conclusão de modo explícito.
- [x] O contrato não fixa transporte, persistência ou permissões Tauri antes das tasks responsáveis.
- [x] A relação entre o contrato canônico e os mocks atuais está documentada.

## Validação

- Revisar este arquivo lado a lado com `README.md`, `PRODUCT.md`, `DESIGN.md`, `SYSTEM-DESIGN.md` e `AGENTS.md`.
- Procurar conflitos entre nomes de estados, capabilities e critérios de UI nos cinco documentos.
- Conferir que nenhum estado terminal pode ser exibido como sucesso parcial ou como dispositivo online.
- Confirmar que `online`, `lastSeen: "agora"` e `in-progress` permanecem identificados como representação mockada até a integração posterior.
- Executar uma revisão de cenários: descoberta sem confiança, pairing recusado, revogação durante transferência, capability ausente, peer offline, retomada após falha, notificação dispensada e comando remoto não autorizado.
- `npm run typecheck`: passou.
- `npm run build`: passou; os mocks e a prévia web continuam compilando sem integração real.
- `cargo check --manifest-path src-tauri/Cargo.toml`: passou.
- A bridge, a persistência, o transporte e os smoke tests de funcionalidade continuam adiados para as tasks responsáveis.

## Dependências desbloqueadas

Após a conclusão desta task, ficam definidos os insumos para:

- TASK 02 — discovery, transporte e ciclo de conexão;
- TASK 03 — threat model, identidade, trust e capabilities;
- TASK 04 — persistência, migrações e retenção;
- TASK 05 — contrato da bridge Rust ↔ Vue;
- TASK 06 — fixtures e testes de transição.
