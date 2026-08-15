# TASK 09 — Implementar comandos e eventos tipados da bridge

Status: concluída; a integração da bridge com produto ainda não hidrata stores nem aciona recursos de rede

## Objetivo

Conectar o contrato da TASK 05 ao runtime Rust sem acoplar a UI às implementações internas. A implementação desta task cria uma bridge mínima, tipada e redigida: requests versionados, respostas de leitura, erros fechados, commands de informação/snapshot seguro, evento de status do runtime, validação de envelopes e um cliente TypeScript centralizado com lifecycle de listeners.

Esta task não implementa discovery, pairing, trust, capabilities, transferências, Clipboard, mídia, histórico conectado ou comandos de efeito. O snapshot ainda informa que o estado de produto não está configurado; os stores Pinia continuam mockados até a TASK 10. `greet` permanece um smoke test separado e não é transformado em command versionado.

## Estado atual

- `TODO.md:92-104` identificava a TASK 09 como a pendência seguinte e separava bridge tipada da integração dos stores da TASK 10.
- A TASK 05 definiu `bridgeContractVersion=1`, envelopes `camelCase`, `requestId`, respostas `success/stale/offline`, erros fechados, eventos namespaced, sequência/deduplicação, ressincronização e fallback web honesto (`docs/tasks/TASK-05-contrato-da-bridge-rust-vue.md:44-234`).
- Antes desta task, `src/composables/useRustBridge.ts:1-21` conhecia diretamente `invoke` e só chamava `greet`; não havia `listen`, cliente central, validação de resposta ou commands de produto.
- Antes desta task, `src-tauri/src/lib.rs:1-50` registrava `greet`, o runtime e o storage, mas nenhum DTO, `Result` serializável ou evento IPC.
- O runtime fornece `RuntimeState::snapshot()` e estados fechados, sem expor `Mutex`, slots ou detalhes de serviço à UI (`src-tauri/src/runtime/mod.rs:140-151,410-455`).
- A TASK 06 já mantém fixtures de `BridgeEvent`/`BridgeError` com versões, sequência, gaps e dados redigidos (`tests/fixtures/bridge.ts:1-64`, `tests/support/fixture-validation.ts:1-33`, `tests/bridge-contract.test.ts:1-43`).
- O storage da TASK 08 está atrás do runtime e não deve atravessar a bridge; `bridge_get_snapshot` não poderá expor SQLite, path, SQL, estado de migration ou conteúdo (`src-tauri/src/storage/mod.rs:1-16`, `docs/tasks/TASK-08-persistencia-local-e-migracoes.md:119-153`).
- A documentação oficial do Tauri confirma que commands recebem tipos `Deserialize`, retornam tipos `Serialize`/`Result`, e que listeners retornam uma `Promise<UnlistenFn>` que só pode ser desmontada depois de resolvida ([Calling Rust from the Frontend](https://v2.tauri.app/develop/calling-rust/), [Calling the Frontend from Rust](https://v2.tauri.app/develop/calling-frontend/)).

## Brainstorm

### Alternativas consideradas

| Alternativa | Avaliação | Decisão |
| --- | --- | --- |
| Commands separados com DTOs fechados | Mantém catálogo revisável, validação explícita e tipos públicos pequenos. | Escolhida |
| Command genérico com `serde_json::Value` | Aceita intenções não catalogadas, enfraquece limites e pode virar bypass de capabilities. | Rejeitada |
| Erro `String` | Pode transportar SQL, path, stack trace ou mensagem de crate e exige parsing de copy no frontend. | Rejeitada |
| Evento de status namespaced | Permite testar emissão e lifecycle sem simular dispositivos ou rede. | Escolhida |
| Emitir `DomainEvent` de produto agora | Não há serviço produtor real; criar fatos artificiais faria a UI acreditar em integração. | Adiada |
| `listen` em cada componente | Duplica listeners em uma SPA e torna cleanup dependente de navegação. | Rejeitada |
| Cliente frontend único com subscribers | Centraliza `invoke`/`listen`, valida eventos e coordena unlisten/deduplicação. | Escolhida |
| Snapshot vazio tratado como sucesso | Confunde ausência de serviço com ausência real de dispositivos. | Rejeitada; retorna `offline` e marca estado como não configurado |
| Fallback web com eventos fake | Contradiz progresso honesto e pode mascarar falhas do shell Tauri. | Rejeitada; `greet` continua demo e listeners são no-op |

### Perguntas que a implementação precisa responder

1. Um payload com versão ou campo desconhecido é rejeitado antes de alcançar o runtime?
2. A resposta Rust contém somente estado público, sem slot, mutex, path, SQL ou erro bruto?
3. O runtime parcial aparece como `offline`/`not-configured`, sem simular dispositivos ou sucesso de produto?
4. O frontend consegue abrir uma única assinatura de evento, deduplicar `eventId`, detectar gap e pedir ressincronização?
5. O fallback web mantém a mesma forma de cliente sem chamar APIs Tauri nem simular eventos?

## Decisões

### 1. Superfície v1 implementada

Foram registrados em um único `generate_handler!`:

- `greet`: exceção legada, sem envelope, preservado como smoke test;
- `bridge_get_info`: leitura versionada com modo `tauri`, versão da bridge/modelo, fase pública do runtime e `productCommandsAvailable=false`;
- `bridge_get_snapshot`: leitura versionada com fase pública e `productState=not-configured`, retornando `offline` enquanto serviços de produto não estiverem ativos.

Não haverá command genérico, SQL, snapshot de dispositivos fabricado, mutation de storage ou command de efeito. A emissão de `pulse:bridge:status` ocorrerá no `setup` depois do start do runtime e conterá somente fase pública, versão, stream, sequência e disponibilidade de commands. O separador `:` é necessário porque o Tauri 2 rejeita pontos em nomes de eventos.

### 2. DTOs Rust ↔ TypeScript

Os DTOs vivem em `src-tauri/src/bridge/mod.rs` e `src/types/bridge.ts`, com nomes `camelCase`, `serde(deny_unknown_fields)` nos requests/eventos e enums fechados. O contrato inclui:

- `BridgeRequest<T>` com `bridgeContractVersion`, `requestId` e payload;
- `BridgeReadResponse<T>` com `requestId`, `status`, `generatedAt`, `observedAt` opcional e `data` opcional;
- `BridgeInfo`, `BridgeSnapshot`, `BridgeEvent<T>` e `BridgeStatusPayload`;
- `BridgeError` com código allowlistado, `retryable`, `messageKey` e `reasonCode` opcional;
- `BridgeMode`, `BridgeReadStatus`, `PublicRuntimePhase` e `ProductState` sem `Debug` ou texto interno atravessando a API.

`modelVersion` continua separado de `bridgeContractVersion`. `protocolVersion` não entra nesses DTOs enquanto a TASK 20 não existir.

### 3. Validação e redaction

- `bridgeContractVersion` deve ser exatamente `1`.
- `requestId` aceita somente IDs ASCII curtos (`A-Z`, `a-z`, números, `.`, `_`, `-`, `:`), entre 1 e 128 caracteres; um ID inválido não é refletido no erro.
- Commands de leitura aceitam somente a webview Tauri principal (`main`); uma origem de janela diferente vira `invalid-request` com `unsupported-window`.
- Payloads de commands usam tipos concretos e rejeitam campos desconhecidos por desserialização.
- Falha de `RuntimeState::snapshot()` vira `runtime-not-ready`, sem `Display` de erro interno.
- Falha de `emit` vira diagnóstico fechado de startup e não atravessa a UI.
- Nenhuma resposta/evento contém `AppHandle`, `Window`, storage, SQL, path, endpoint, token, chave, payload remoto ou mensagem bruta de crate.

### 4. Cliente frontend e listeners

`src/bridge/client.ts` é a única fronteira TypeScript com `invoke`/`listen`; `useRustBridge.ts` apenas expõe o cliente e preserva `greet`.

- `getInfo()` e `getSnapshot()` geram `requestId` local e enviam envelopes `camelCase`.
- No web preview, `greet` mantém `isDemo=true`; leituras retornam resposta local `web-preview`/`offline` com estado `not-configured`, sem chamar Tauri.
- `listenStatus()` e `listenDomainEvents()` são assinaturas centralizadas; múltiplos consumidores compartilham um listener nativo e cada unsubscribe remove somente seu subscriber.
- A Promise de `listen` é guardada; cleanup aguarda sua resolução antes de chamar `unlisten`.
- O cliente valida formato, versão, `modelVersion`, `streamId`, `sequence` e `eventId`. Evento duplicado ou atrasado é ignorado; gap ou troca de stream aciona callback de ressincronização e não é entregue como estado atual.
- Não haverá listener local dentro de views ou stores nesta task; a TASK 10 decidirá como ligar o cliente à fonte de estado.

### 5. Testes

Rust cobre serialização `camelCase`, campos desconhecidos, request/version validation, erro redigido, mapeamento do snapshot e formato do evento de status. TypeScript cobre request/response, fallback web, validação de eventos, deduplicação, gap, mudança de stream, subscribers e cleanup sem chamar `unlisten` cedo.

Os testes não abrem janela, rede, keyring ou banco do usuário. A emissão real no `setup` é validada por compilação e por helper puro; nenhum teste cria dispositivo, trust ou transferência falsa.

## Plano de implementação

1. Adicionar `serde` com derive e `serde_json` como dependência de teste Rust.
2. Criar `src-tauri/src/bridge/mod.rs` com constantes, DTOs, erros fechados, validators, commands e emitter de status.
3. Registrar o módulo e os commands no `lib.rs`; emitir status após o runtime iniciar, preservando `greet`.
4. Criar `src/types/bridge.ts`, cliente centralizado e validação/sequence tracker no frontend; adaptar `useRustBridge.ts` sem mudar o comportamento de `greet`.
5. Adicionar testes Rust e TypeScript para versões, desconhecidos, erros, fallback, listeners, deduplicação e gaps.
6. Atualizar `TODO.md`, `SYSTEM-DESIGN.md`, `README.md`, `PRODUCT.md` e este plano somente depois da implementação e da revisão do diff.
7. Executar `npm run typecheck`, `npm test`, `npm run test:rust`, `npm run build`, `cargo check`, `cargo fmt --check` e `git diff --check`.

## Execução paralela

A investigação foi separada em dois recortes sem escrita sobreposta:

- **Contrato e segurança:** cruzamento da TASK 05, fixtures da TASK 06, runtime/storage atuais e modelos de domínio para fixar DTOs públicos, erros, estado parcial e dados proibidos.
- **APIs do framework:** consulta à documentação oficial do Tauri sobre commands, `Serialize`/`Deserialize`, `Result`, eventos JSON, `listen` e `unlisten`.

A implementação será sequenciada por fronteira: DTO/commands Rust, cliente TypeScript, testes e documentação. Não há paralelismo real adicional que justifique editar `lib.rs`, o cliente e os contratos simultaneamente.

## Integração

- A TASK 05 continua sendo a fonte do contrato; esta task transforma somente o subconjunto de infraestrutura seguro em código.
- A TASK 06 fornece fixtures e cenários negativos; a bridge não reutiliza mocks de dispositivos/transferências.
- A TASK 07 fornece `RuntimeState`; a bridge expõe apenas fase pública e nunca seus slots/mutex.
- A TASK 08 continua atrás de serviços; erros de storage não atravessam a bridge como SQL/path.
- A TASK 10 poderá consumir `getSnapshot`, status e eventos por um adaptador de store; até lá os stores permanecem mockados.
- As TASKS 11+ acrescentarão commands/eventos específicos somente com serviços reais, capability, origem, limites e estado confirmado.

## Critérios de conclusão

- [x] `serde`/DTOs Rust e tipos TypeScript compartilham envelope, nomes, versões e códigos fechados.
- [x] `bridge_get_info` e `bridge_get_snapshot` são registrados e retornam somente estado público/redigido.
- [x] `pulse:bridge:status` é emitido no startup com stream/sequência e sem dados de infraestrutura.
- [x] Requests rejeitam versão, ID, payload e campos desconhecidos inválidos antes do runtime.
- [x] Erros de command são serializáveis, fechados, retryable e sem mensagem bruta.
- [x] Cliente frontend centraliza `invoke`/`listen`, preserva fallback web e não simula sucesso de produto.
- [x] Listeners compartilham registro nativo, aguardam a Promise de `listen`, removem subscribers e fazem cleanup correto.
- [x] Eventos duplicados/atrasados são ignorados; gaps e streams novos exigem ressincronização.
- [x] `greet`, storage, runtime, mocks e capabilities atuais continuam válidos.
- [x] Testes Rust/Vue e todos os comandos de validação passam.

## Validação

### Evidência local revisada

- `TODO.md:92-104` — objetivo, dependências, critérios e validação da TASK 09.
- `docs/tasks/TASK-05-contrato-da-bridge-rust-vue.md:44-234,255-306` — envelopes, catálogo, eventos, erros, validação, preview e cenários.
- `docs/tasks/TASK-06-base-de-testes-e-fixtures.md:112-142,164-183` — fixtures, peers, casos negativos e isolamento dos testes.
- `docs/tasks/TASK-07-runtime-de-servicos-rust.md:112-151` — fronteira pública do runtime e preservação de `greet`.
- `src-tauri/src/lib.rs:1-50`, `src-tauri/src/runtime/mod.rs:140-151,410-455` — ponto de registro, lifecycle e snapshot interno.
- `src-tauri/src/bridge/mod.rs:1-465`, `src-tauri/src/lib.rs:1-50` — DTOs, validação de request/origem, timestamps UTC, commands, evento de status e registro Tauri implementados.
- `src/bridge/client.ts:1-554`, `src/composables/useRustBridge.ts:1-17` — cliente centralizado, fallback web, validação, sequência e lifecycle de listeners.
- `tests/bridge-client.test.ts:1-244`, `tests/bridge-contract.test.ts:1-43` — contratos de transporte, erros, fallback, subscribers, deduplicação, gaps e cleanup.

### Fontes primárias consultadas

- [Tauri — Calling Rust from the Frontend](https://v2.tauri.app/develop/calling-rust/) — commands, desserialização, serialização e erros.
- [Tauri — Calling the Frontend from Rust](https://v2.tauri.app/develop/calling-frontend/) — eventos, `listen`, `unlisten` e cleanup.

### Matriz de validação

| Cenário | Resultado exigido |
| --- | --- |
| Request válido | Command recebe envelope e retorna resposta `camelCase` versionada. |
| Versão incompatível | Erro `unsupported-contract-version`; runtime não é chamado. |
| Campo desconhecido | Desserialização/validação falha como `invalid-request`; nenhum dado bruto é propagado. |
| Runtime parcial | Info/snapshot exibem fase pública e `offline/not-configured`, sem dispositivos falsos. |
| Erro interno | Bridge retorna código/copy key fechados, sem path/SQL/mensagem de crate. |
| Startup | Evento `pulse:bridge:status` tem versão, stream, sequência e payload redigido. |
| Evento duplicado | Cliente entrega no máximo uma vez por `eventId`. |
| Gap/out-of-order | Cliente não entrega como atual e solicita ressincronização. |
| Web preview | `greet` segue demo; commands/listeners de produto não chamam Tauri nem simulam sucesso. |
| Cleanup SPA | `unlisten` só ocorre após a Promise de `listen` resolver e não há listener nativo duplicado. |

### Execução realizada

- `npm run typecheck` — passou.
- `npm test` — passou: 5 arquivos, 22 testes.
- `npm run test:rust` — passou: 26 testes Rust, 0 falhas.
- `npm run build` — passou.
- `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` — passou após aplicar o formatter.
- `cargo check --manifest-path src-tauri/Cargo.toml` — passou.
- `git diff --check` — passou.
- `npm run tauri:dev` — shell iniciou e permaneceu ativo sem panic de setup; o evento inicial `pulse:bridge:status` foi aceito após o runtime iniciar. O processo foi interrompido manualmente depois da confirmação. Não foram adicionados serviços de produto, rede ou capabilities.
