# TASK 05 — Definir o contrato da bridge Rust ↔ Vue

Status: contrato definido; implementação dos comandos, eventos e adaptadores fica para as TASKS 07, 09 e 10.

## Objetivo

Definir uma fronteira IPC estável entre a UI Vue e os serviços Rust do Pulse. O contrato deve transportar intenções, estados observados, eventos de domínio e erros sem expor sockets, QUIC, mDNS, keyring, SQLite, paths completos, segredos ou detalhes de implementação.

Esta task é de contrato e decisão. Ela não adiciona comandos de produto, eventos ativos, dependências de serialização, serviços Rust, listeners, persistência ou adaptação dos mocks atuais. O command `greet` continua sendo somente o smoke test da fundação.

## Estado atual

- O produto é uma fundação navegável `0.1.0`; a tela de Configurações chama apenas `greet`, enquanto dispositivos, transferências e estados de produto continuam mockados (`PRODUCT.md:35-63`, `src/views/SettingsView.vue:1-74`).
- A bridge atual detecta `window.__TAURI_INTERNALS__`, chama `invoke<string>("greet", { name })` no Tauri e devolve uma mensagem demonstrativa na prévia web (`src/composables/useRustBridge.ts:1-21`).
- No estado anterior à TASK 09, o único command registrado era `greet`; os eventos, listeners e DTOs previstos aqui foram implementados posteriormente na TASK 09 (`src-tauri/src/lib.rs:1-20`, `src-tauri/Cargo.toml:1-19`).
- Os modelos canônicos TypeScript e Rust já possuem entidades, estados, transições, `DomainEvent` e `DOMAIN_MODEL_VERSION`, mas não conhecem Tauri ou uma codificação IPC (`src/types/index.ts:1-18,391-435`, `src-tauri/src/domain/mod.rs:1-8,748-778`).
- A arquitetura exige que a UI dependa de modelos/eventos estáveis e que a bridge valide entrada antes de encaminhar ao domínio, sem expor transporte (`SYSTEM-DESIGN.md:123-146`).
- A TASK 03 exige que a UI nunca receba chave privada, token, transcript, payload bruto, path completo ou detalhes de keyring; a TASK 04 mantém segredo fora do SQLite e acessível somente por serviços tipados (`docs/tasks/TASK-03-threat-model-identidade-trust-capabilities.md`, `docs/tasks/TASK-04-persistencia-migracoes-e-retencao-local.md`).
- O Tauri 2 aceita argumentos desserializáveis, retornos serializáveis e `Result` com erro serializável para commands. O sistema de eventos é dinâmico, assíncrono, sem retorno e baseado em JSON; `listen` devolve um `unlisten` assíncrono que deve ser chamado quando o escopo termina. Para fluxo contínuo de maior volume, a documentação recomenda Channels em vez de eventos ([Calling Rust from the Frontend](https://v2.tauri.app/develop/calling-rust/), [Calling the Frontend from Rust](https://v2.tauri.app/develop/calling-frontend/)).

## Brainstorm

### Alternativas consideradas

| Alternativa | Avaliação | Decisão |
| --- | --- | --- |
| `invoke` genérico com `command: string` e `payload: unknown` | Flexível no início, mas remove a lista de intenções permitidas, enfraquece validação, facilita acoplamento da UI e torna erros difíceis de revisar. | Rejeitada |
| Cada componente Vue chama `invoke`/`listen` diretamente | Espalha nomes de commands, cria listeners duplicados em uma SPA e permite que detalhes IPC virem dependência de tela. | Rejeitada |
| Somente polling por snapshots | Perde mudanças de presença e progresso, aumenta latência e não representa bem eventos de domínio. | Rejeitada |
| Somente eventos globais sem snapshot inicial | Pode perder eventos durante inicialização/reconexão e não fornece hidratação determinística. | Rejeitada |
| Eventos Tauri para cada atualização de alto volume | Eventos são JSON e não oferecem a semântica de stream necessária para progresso frequente. | Rejeitada como política geral; Channels ficam disponíveis para streaming futuro |
| Erro como `String` livre | Vaza detalhes internos, não permite tratamento determinístico e dificulta copy acessível na UI. | Rejeitada para commands de produto |
| DTOs que espelham `quinn`, sockets, keyring ou SQLite | Faz a UI depender de infraestrutura e transforma uma decisão técnica em API pública. | Rejeitada |
| Fallback web que simula sucesso nativo | Faz a prévia parecer integração funcional e contradiz a regra de progresso honesto. | Rejeitada |

### Perguntas que o contrato precisa responder

1. Como uma intenção da UI é correlacionada com sua resposta sem prometer que aceitação equivale a efeito concluído?
2. Como um snapshot inicial e eventos incrementais convivem sem perder mudanças ou aplicar eventos fora de ordem?
3. Como diferenciar loading local, sucesso de IPC, estado stale/offline e erro de uma operação?
4. Como evoluir bridge, modelo de domínio e protocolo de peer sem downgrade silencioso?
5. Como manter a prévia web útil para layout sem declarar networking, persistência ou serviços reais?

## Decisões

### 1. Fronteira e responsabilidades

O contrato v1 terá quatro camadas explícitas:

1. **UI Vue:** solicita intenções por um cliente tipado, renderiza estados e aplica copy; não importa `invoke`, `listen`, `AppHandle`, `WebviewWindow`, `quinn`, `SocketAddr`, SQLite ou keyring.
2. **Adaptador frontend da bridge:** concentra `invoke`/`listen`, gera `requestId`, valida envelopes recebidos, converte falhas de transporte IPC para `BridgeError` e oferece uma assinatura descartável.
3. **Bridge Tauri:** recebe DTOs fechados, valida contrato e autorização de entrada, chama serviços Rust e serializa somente respostas/eventos públicos.
4. **Serviços Rust:** aplicam invariantes de domínio, trust, capability, persistência e transporte. A bridge não substitui essas validações nem recebe autorização implícita da UI.

O adaptador será a única fronteira autorizada a conhecer a API Tauri. Stores poderão consumi-lo quando a TASK 10 conectar estado real; a TASK 09 fará o registro efetivo dos commands e eventos.

### 2. Versões independentes

O contrato separa três versões:

| Versão | Fonte | Uso |
| --- | --- | --- |
| `bridgeContractVersion` | bridge ↔ Vue | Forma dos commands, envelopes, erros e eventos IPC |
| `modelVersion` | `DOMAIN_MODEL_VERSION` | Forma e semântica dos modelos/eventos de domínio |
| `protocolVersion` | TASKS 20–22 | Mensagens entre peers e negociação de transporte |

O valor inicial de `bridgeContractVersion` é `1`, independente de `modelVersion = 1`. Uma mudança incompatível abre uma nova versão major do contrato; não há tentativa silenciosa de converter payload desconhecido. Mudanças aditivas devem manter campos opcionais e ser aceitas somente quando não alterarem invariantes.

Todo request de produto informa `bridgeContractVersion`; toda resposta e evento informa a versão suportada. O primeiro command de informação negociará as versões suportadas antes de habilitar serviços. `greet` permanece uma exceção deliberadamente não versionada e fora do catálogo de produto.

### 3. Envelope de requests e respostas

Commands de produto usarão um envelope comum. Os nomes abaixo são contratos lógicos para a TASK 09; o registro Rust pode organizar funções em módulos sem expor essa organização à UI.

```ts
type BridgeRequest<T> = {
  bridgeContractVersion: 1;
  requestId: string;
  payload: T;
};

type BridgeReadResponse<T> = {
  bridgeContractVersion: 1;
  requestId: string;
  status: "success" | "stale" | "offline";
  generatedAt: UtcTimestamp;
  observedAt?: UtcTimestamp;
  data?: T;
};
```

`requestId` correlaciona uma chamada IPC e um erro; não é segredo, não é prova de identidade e não substitui um ID de sessão ou de operação. Operações com efeito devem usar o ID de domínio definido por sua própria task e não podem depender de retry cego de uma chamada IPC.

`status: "stale"` e `status: "offline"` são resultados observáveis de leitura quando o último estado conhecido pode ser exibido ou quando não há observação atual. Não significam trust revogado. A resposta pode omitir `data` quando não houver estado seguro para exibir.

Para commands de intenção, a resposta positiva confirma somente que a intenção foi validada/aceita pelo serviço. O efeito posterior chega por `DomainEvent`; uma operação não deve ser marcada como concluída apenas porque o `invoke` resolveu.

O ciclo `loading` é local ao cliente Vue: começa antes de `invoke` e termina quando a Promise resolve ou rejeita. Ele não será enviado como estado de domínio pelo Rust.

### 4. Catálogo lógico inicial de commands

O catálogo abaixo define famílias, entradas e resultados; a implementação e o detalhamento de cada recurso pertencem às tasks dependentes.

| Command lógico | Natureza | Entrada mínima | Saída |
| --- | --- | --- | --- |
| `bridge_get_info` | leitura | envelope sem payload | versão da bridge/modelo, modo (`tauri`/`web-preview`) e capacidades do runtime sem detalhes de transporte |
| `bridge_get_snapshot` | leitura | escopo opcional validado | snapshot inicial de dispositivos, presença, trust, capabilities, transferências, histórico/notificações permitidos |
| `device_refresh` | intenção local | escopo de atualização | aceite da solicitação; mudanças chegam como eventos |
| `pairing_start` | intenção com efeito | candidato/identidade apresentados e parâmetros limitados | `PairingSession` criada ou erro; nunca trust implícito |
| `pairing_decide` | confirmação explícita | sessão, decisão e prova de confirmação permitida | sessão atualizada; trust só após as regras bilaterais da TASK 16 |
| `capability_decide` | autorização | dispositivo, capability, direção e decisão | `CapabilityGrant` atualizado após validação do owner |
| `transfer_create` / `transfer_control` | sessão de recurso | manifesto/ação já validados pela task de arquivos | `TransferSession` observável; conclusão via evento e confirmação |
| `light_content_send` / `clipboard_request` | conteúdo leve | conteúdo explícito e capability aplicável | aceite ou erro; conteúdo não aparece em histórico por implicação |
| `media_read` / `remote_command_request` | leitura/ação limitada | dispositivo e ação canônica allowlistada | estado/aceite; resultado confirmado por evento |

Commands não podem aceitar um objeto de infraestrutura genérico, `serde_json::Value` livre, nome de tabela, SQL, endereço de rede, token, chave, path completo ou comando shell. A adição de um command exige capability, origem, limites, erro e evento correspondentes documentados na task do recurso.

### 5. Eventos e sincronização

O contrato terá eventos nomeados e namespaced:

| Evento | Conteúdo | Regra |
| --- | --- | --- |
| `pulse.bridge.status` | estado do runtime/bridge e motivo público | não contém erro bruto nem detalhe de infraestrutura |
| `pulse.domain.event` | `DomainEvent` em envelope de bridge | fonte incremental de mudança de domínio |
| `pulse.domain.snapshot-invalidated` | motivo e sequência do stream | solicita novo snapshot quando houver gap, evento inválido ou ressincronização necessária |

O payload de evento será:

```ts
type BridgeEvent<T> = {
  bridgeContractVersion: 1;
  streamId: string;
  sequence: number;
  eventId: DomainEventId;
  emittedAt: UtcTimestamp;
  modelVersion: typeof DOMAIN_MODEL_VERSION;
  payload: T;
};
```

`streamId` muda quando o runtime reinicia; `sequence` é monotônica dentro do stream. O adaptador deduplica `eventId`, detecta salto de sequência e solicita snapshot antes de expor uma leitura como atual. A ordem de chegada de callbacks assíncronos não é tratada como ordem de domínio; o consumidor precisa aplicar a sequência/eventual ressincronização.

Eventos são para fatos pequenos e mudanças de estado. Progresso de alta frequência, bytes de arquivo e conteúdo grande não serão enviados por `pulse.domain.event`; a task responsável poderá adotar Tauri Channels com limites e backpressure. O contrato nunca transforma um evento JSON em transporte de arquivo.

Listeners serão registrados uma vez pelo cliente de bridge no ciclo de vida da aplicação, preferencialmente após criar a assinatura e antes de pedir `bridge_get_snapshot`. O cliente deve:

- guardar o `Promise<UnlistenFn>` retornado por `listen`;
- só chamar `unlisten` depois que a Promise resolver;
- remover listeners ao destruir o escopo e durante HMR/reinicialização do cliente;
- impedir registro duplicado no mesmo `streamId`/cliente;
- ignorar ou encaminhar para ressincronização eventos com versão, tipo ou payload inválidos;
- não depender da navegação do Vue Router para limpar listeners, pois a aplicação é uma SPA;
- preferir o webview atual a eventos globais quando a UI tiver mais de uma janela.

### 6. Erros públicos e estados

Commands de produto retornarão `Result<Success, BridgeError>` em Rust. `BridgeError` será serializável e fechado:

```ts
type BridgeErrorCode =
  | "invalid-request"
  | "unsupported-contract-version"
  | "runtime-not-ready"
  | "not-found"
  | "already-resolved"
  | "trust-required"
  | "capability-denied"
  | "peer-offline"
  | "transport-unavailable"
  | "storage-unavailable"
  | "timeout"
  | "canceled"
  | "conflict"
  | "internal";

type BridgeError = {
  bridgeContractVersion: 1;
  requestId: string;
  code: BridgeErrorCode;
  retryable: boolean;
  messageKey: string;
  reasonCode?: string;
};
```

`messageKey` aponta para copy localizável; não é texto de exceção vindo de Rust. `reasonCode` deve ser uma chave não sensível e limitada por catálogo. Stack trace, SQL, path, URL, endpoint, nome de tabela, chave, token, payload remoto e mensagem de crate nunca atravessam a bridge.

O mapeamento de estado é deliberadamente separado:

| Camada | Estados |
| --- | --- |
| Ciclo local do request | `idle`, `loading`, `success`, `error` |
| Leitura de estado observado | `success`, `stale`, `offline` |
| Presença de dispositivo | `unknown`, `online`, `stale`, `offline` |
| Trust | `unpaired`, `trusted`, `revoked` |
| Resultado de ação | resposta aceita, evento em andamento e evento concluído/negado/falho |

`offline`, `stale`, `denied` e `revoked` não são sinônimos. Uma leitura offline pode retornar `lastSeenAt`; uma operação contra peer offline rejeita com `peer-offline`; capability negada não implica ausência de rede; trust revogado não pode ser reativado por heartbeat.

### 7. Validação de entrada e de eventos

Antes de encaminhar um command, a bridge deve:

- aceitar somente commands registrados e envelopes com `bridgeContractVersion` suportada;
- desserializar DTOs com campos conhecidos e rejeitar campos desconhecidos quando o payload for de intenção;
- validar presença, tamanho, formato e charset dos IDs, nomes, reason codes, limites numéricos e strings opcionais;
- conferir que o request pertence à janela ativa e que a operação é permitida para o dispositivo, direção, trust e capability corretos;
- validar novamente invariantes no serviço Rust; validação feita na UI é apenas ergonomia;
- limitar payloads e frequência antes de alcançar serviço de recurso;
- redigir ou excluir dados sensíveis antes de gerar resposta, evento, log ou notificação;
- rejeitar evento com versão desconhecida, `modelVersion` incompatível, `eventType` fora do catálogo, sequência inválida ou payload que não corresponde ao tipo do evento.

O envelope não será um escape hatch para `Record<string, unknown>` em commands. O `payload` permissivo já existente em `DomainEvent` não concede à UI permissão para criar eventos; somente serviços Rust podem produzir fatos de domínio.

### 8. Prévia web

O cliente expõe o mesmo tipo de interface nos dois ambientes e informa um `BridgeMode`:

- `tauri`: commands/eventos nativos podem ser usados quando registrados e autorizados;
- `web-preview`: `greet` retorna a mensagem de demonstração existente, listeners de produto são no-op e commands de produto retornam `preview-only` por um erro local tipado ou entregam fixture explicitamente marcada como demo quando a task autorizar.

A prévia web não emite eventos falsos, não simula online, pairing, trust, transferência concluída, persistência ou sucesso remoto. Os fixtures visuais atuais continuam nos stores até a TASK 10, com sua marcação demonstrativa. A detecção atual por `window.__TAURI_INTERNALS__` permanece o mecanismo do shell; `withGlobalTauri` continua desabilitado.

### 9. Dados que nunca cruzam a bridge

Não fazem parte de nenhum DTO público:

- chave privada, seed, segredo de recuperação, token de sessão, nonce de protocolo, transcript ou assinatura;
- socket, `SocketAddr`, porta, registro TXT, interface de rede, objeto QUIC ou detalhe de mDNS;
- conexão SQLite, SQL, path de banco, lock, erro estrutural ou segredo do Secret Service;
- path local completo ou conteúdo de arquivo sem a capability e o fluxo de seleção próprios;
- conteúdo de Clipboard, texto/link, payload remoto ou erro bruto em histórico/notificação;
- `AppHandle`, `WebviewWindow`, label de janela ou referência a processo.

## Plano de implementação

Esta task define o contrato e não implementa os itens abaixo:

1. A TASK 07 criará o estado compartilhado e o ciclo de vida de serviços sem alterar os DTOs públicos.
2. A TASK 09 adicionará `serde`, erros serializáveis, registro dos commands, emissão dos eventos, validação e testes de contrato Rust/Vue.
3. A TASK 06 criará fixtures de envelopes válidos, versões incompatíveis, gaps de sequência, payloads inválidos e erros redigidos.
4. A TASK 10 criará o adaptador/store que hidrata snapshot, aplica eventos e mantém fixtures somente no modo de desenvolvimento.
5. As tasks de pairing, persistência, transporte e recursos acrescentarão comandos/eventos específicos sem abrir acesso genérico à UI.

## Execução paralela

A investigação foi separada em dois recortes sem escrita sobreposta:

- **Contrato local:** cruzamento de `PRODUCT.md`, `DESIGN.md`, `SYSTEM-DESIGN.md`, TODO, TASKS 01/02/03/04, tipos TypeScript/Rust, composable, store e configuração Tauri para separar o que existe do que é direção futura.
- **Comportamento Tauri:** consulta à documentação oficial do Tauri 2 sobre commands, `Result` serializável, eventos JSON sem tipagem forte, `listen`/`unlisten` e Channels para streams.

A consolidação das versões, envelopes, erros, lifecycle e fallback web foi feita sequencialmente neste plano. Não houve implementação paralela nem alteração em código.

## Integração

- A TASK 07 deve manter a bridge capaz de reportar runtime parcial ou não configurado sem fabricar serviços disponíveis.
- A TASK 08 deve permanecer atrás de APIs de serviço; SQL, corrupção e paths de storage não atravessam a bridge.
- A TASK 09 deve registrar commands em um único `generate_handler!`, aplicar validação no Rust, emitir somente eventos allowlistados e preservar `greet` como smoke test.
- A TASK 10 deve consumir snapshot/eventos por um adaptador único, tratar gaps como ressincronização e manter os mocks explicitamente demonstrativos.
- As TASKS 11–22 devem preservar a separação entre candidato, presença, pairing, trust, capability, transporte e estado de operação.
- As TASKS 23 em diante devem usar comandos tipados e efeitos confirmados; aceitação IPC nunca equivale a arquivo recebido, Clipboard escrito, mídia controlada ou comando concluído.
- `SYSTEM-DESIGN.md` deve registrar a decisão do contrato sem alterar o resumo de que nenhum command/evento de produto está implementado.

## Critérios de conclusão

- [x] A fronteira UI → adaptador → bridge → serviços Rust está definida sem expor transporte ou armazenamento.
- [x] Commands, envelopes, catálogo lógico inicial, eventos e payloads têm regras de forma, correlação e resultado.
- [x] `bridgeContractVersion`, `modelVersion` e `protocolVersion` estão separados, com comportamento para incompatibilidade.
- [x] Erros têm códigos fechados, retryability e copy por chave, sem texto bruto ou dados sensíveis.
- [x] Loading, success, stale, offline, trust e capability têm semânticas distintas e textualmente representáveis.
- [x] Validação de entrada, autorização no Rust, limites, eventos inválidos e ressincronização estão definidos.
- [x] Lifecycle de listeners cobre Promise de `unlisten`, SPA, duplicação, gaps e webview alvo.
- [x] A prévia web é compatível em forma, mas não simula integração nativa nem sucesso de produto.
- [x] O command `greet` continua explicitamente como smoke test e nenhuma implementação de produto foi adicionada.

## Validação

### Evidência revisada

- `PRODUCT.md:35-63` e `DESIGN.md:28-37,100-110` confirmam fundação/mock, teste da bridge e estados textuais atuais.
- `src/composables/useRustBridge.ts:1-21`, `src/stores/app.ts:1-32` e `src/views/SettingsView.vue:1-74` confirmam a única chamada IPC e o ciclo local `idle/loading/success/error`.
- `src-tauri/src/lib.rs:1-20`, `src-tauri/Cargo.toml:1-19` e `src-tauri/capabilities/default.json:1-7` confirmam que não há commands de produto, eventos, serialização adicional ou capabilities extras.
- `src/types/index.ts:26-85,391-435,550` e `src-tauri/src/domain/mod.rs:1-8,748-778` confirmam versões, estados/eventos e ausência de acoplamento IPC nos modelos puros.
- `SYSTEM-DESIGN.md:78-93,123-146` e as decisões das TASKS 02–04/03 confirmam a fronteira de camadas, o limite do transporte, o armazenamento e os dados que não podem ser expostos.
- A documentação oficial do Tauri 2 confirma `serde::Deserialize`/`Serialize` para commands, `Result` serializável para erros, eventos JSON sem tipagem forte, limpeza via `unlisten` e Channels para streaming.

### Cenários de contrato

| Cenário | Resultado exigido |
| --- | --- |
| Prévia web chama `greet` | Retorna demo marcada; não afirma bridge Rust ativa. |
| Prévia web chama command de produto | Retorna `web-preview`/`preview-only` ou fixture demo explicitamente marcada; nunca efeito real. |
| UI envia versão incompatível | Rejeita com `unsupported-contract-version`; nenhum serviço é chamado. |
| Payload tem campo desconhecido ou tamanho inválido | Rejeita com `invalid-request`; não registra conteúdo bruto. |
| Snapshot perde presença atual | Retorna `stale`/`offline` conforme o estado observável; não revoga trust. |
| Peer está offline durante ação | Rejeita com `peer-offline`; não reporta operação concluída. |
| Command é aceito, mas efeito ainda não terminou | Resposta confirma aceitação; evento posterior confirma andamento/resultado. |
| Evento chega duplicado | Deduplica por `eventId`; não aplica a mutação duas vezes. |
| Evento tem gap ou versão desconhecida | Solicita snapshot; não expõe estado como atual. |
| Componente de rota é desmontado | O cliente não cria listener local; assinatura central permanece ou é descartada pelo lifecycle, sem duplicação. |
| Erro interno Rust contém path/SQL/token | Bridge redige para código e `messageKey`; segredo nunca chega à UI/log/histórico. |

### Validação de implementação

Esta task não altera runtime ou dependências. As validações de código ficam para a TASK 09; após a atualização documental, devem continuar passando `npm run typecheck`, `npm run build` e `cargo check --manifest-path src-tauri/Cargo.toml`, confirmando que `greet`, os mocks e a prévia web permanecem intactos.
