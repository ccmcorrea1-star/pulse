# TASK 10 — Integrar modelos reais ao estado do Vue

Status: concluída; a bridge de infraestrutura está integrada ao bootstrap Vue, mas ainda não há snapshot de produto para dispositivos ou transferências.

## Objetivo

Substituir o acoplamento direto da UI aos mocks por stores com fonte de estado explícita, conectados ao cliente tipado da bridge. A task deve hidratar o estado de infraestrutura no início da aplicação, acompanhar status/eventos da bridge e preservar fixtures visuais somente em desenvolvimento, sempre com copy honesta.

Esta task não implementa discovery, presença real, pairing, trust, capabilities, networking, transferências, Clipboard, mídia ou novos commands Rust. Como o snapshot atual informa `productState: not-configured`, os stores de dispositivos e transferências não podem apresentar dados reais; no modo de desenvolvimento eles poderão continuar exibindo fixtures rotuladas.

## Estado atual

- `TODO.md:99-105` define a TASK 10 como a próxima etapa após a bridge tipada e exige fonte de estado explícita, estados de loading/erro/vazio/stale e fixtures limitadas ao desenvolvimento.
- `src/bridge/client.ts` é a única fronteira TypeScript com `invoke`/`listen` e fornece `getInfo`, `getSnapshot`, status e eventos de domínio com validação, deduplicação e ressincronização.
- `src/types/bridge.ts` define `BridgeInfo`, `BridgeSnapshot`, `BridgeReadResponse`, `BridgeStatusEvent` e `ProductState`; o snapshot atual só admite `productState: not-configured`.
- `src/stores/app.ts` conhece apenas o teste demonstrativo `greet`; não hidrata a bridge nem mantém o estado observado do runtime.
- `src/stores/devices.ts` e `src/stores/transfers.ts` inicializam diretamente arrays de `MockDevice`/`MockTransfer`; componentes e views também importam esses tipos mockados.
- `src/main.ts` monta Pinia e o router sem inicialização coordenada da bridge.
- `src-tauri/src/bridge/mod.rs` confirma que os commands atuais expõem somente infraestrutura pública e não existem comandos de produto.
- `docs/tasks/TASK-05-contrato-da-bridge-rust-vue.md` determina que listeners devem ser registrados antes do snapshot, que gaps exigem ressincronização e que a prévia web não pode simular sucesso remoto.

## Brainstorm

| Alternativa | Avaliação | Decisão |
| --- | --- | --- |
| Cada view chama `getSnapshot` e registra listeners | Duplica requests/listeners e acopla telas ao IPC. | Rejeitada |
| Stores continuam com mocks como estado primário | Mantém a UI sem fonte observável e pode insinuar produto ativo. | Rejeitada |
| Fabricar dispositivos/transferências a partir do snapshot de infraestrutura | Contradiz `not-configured` e o progresso honesto. | Rejeitada |
| Store global de bootstrap da bridge + stores de coleção com origem explícita | Centraliza lifecycle e permite que fixtures sejam uma fonte separada e rotulada. | Escolhida |
| Remover todos os fixtures imediatamente | Quebra a prévia visual e não entrega dados reais enquanto as tasks de produto não existem. | Rejeitada |
| Aplicar todo `DomainEvent` desconhecido diretamente nos arrays | Seria uma interpretação insegura de payload sem contrato de produto. | Rejeitada; eventos só atualizam telemetria/resync nesta task |

## Decisões

1. `app` terá um bootstrap idempotente que registra listeners, consulta info/snapshot e expõe `bridgeMode`, `runtimePhase`, `productState`, estado de sincronização, último evento e erro público.
2. O bootstrap será iniciado uma vez em `src/main.ts`; o cliente de bridge continuará sendo a única camada que conhece Tauri.
3. Status de bridge e eventos de domínio serão observados pelo app store. Um gap ou payload inválido acionará novo snapshot; nenhum evento sem DTO de produto alterará dispositivos ou transferências.
4. Stores de dispositivos e transferências passarão a expor itens de apresentação próprios, `source` (`development-fixture` ou `empty`) e estado de sincronização. Os tipos `MockDevice`/`MockTransfer` ficarão confinados à criação das fixtures.
5. Fixtures só serão carregadas quando `import.meta.env.DEV` for verdadeiro. O build de produção começará com coleções vazias e copy de estado não configurado.
6. A UI exibirá contagens derivadas dos stores e manterá a distinção entre bridge disponível, produto não configurado e fixture visual. Nenhuma ação de transferência será habilitada.
7. O teste `greet` continuará separado como smoke test; erros exibidos pelo store serão reduzidos a chaves/códigos públicos.

## Plano de implementação

1. Criar tipos de apresentação para itens de dispositivo/transferência e estados de fonte/sincronização.
2. Refatorar os stores de dispositivos e transferências para encapsular fixtures, origem, vazio e status observado.
3. Expandir o app store com bootstrap da bridge, listeners, snapshot, resync e estado público redigido.
4. Iniciar o bootstrap no ponto de entrada e adaptar Settings, Home, sidebar, lista de dispositivos e detalhes para consumir os novos estados sem importar tipos mockados.
5. Adicionar testes Vitest para bootstrap web, estado `not-configured`, fonte de fixtures, contagens e resync/cleanup observável.
6. Atualizar `TODO.md`, `SYSTEM-DESIGN.md`, `PRODUCT.md` e este plano com o estado real somente após revisão e validação.

## Execução paralela

- **Contrato e lifecycle:** investigação dos contratos TASK 05/09, do cliente da bridge e do bootstrap Tauri; ownership de `app.ts`, `main.ts` e testes de bridge.
- **Apresentação e stores:** investigação dos tipos canônicos, mocks atuais, componentes e views; ownership de stores, tipos de apresentação e UI.

As alterações serão integradas sequencialmente porque o app store define o estado que a UI consumirá. Não há paralelismo de escrita sobre os mesmos arquivos.

## Integração

- TASK 09 fornece o cliente, snapshot, status e eventos; TASK 10 não amplia o catálogo de commands.
- TASKS 11–14 poderão substituir a fonte `empty`/`development-fixture` por dados de discovery/presença reais mediante novos DTOs e eventos.
- TASKS 24–28 poderão substituir as fixtures de transferência por sessões reais sem alterar a semântica de `source` e dos estados visíveis.
- O runtime Rust e o storage permanecem sem dados de produto atravessando a bridge nesta task.

## Critérios de conclusão

- [x] A inicialização da aplicação hidrata info/snapshot por um único store e registra listeners uma única vez.
- [x] Estados de loading, erro, vazio, stale e offline são representados sem confundir ausência de produto com ausência de dispositivos.
- [x] Gaps/eventos inválidos acionam ressincronização e não alteram coleções com payload não tipado.
- [x] Dispositivos e transferências não dependem diretamente de `MockDevice`/`MockTransfer` fora do boundary de fixtures.
- [x] Fixtures aparecem somente em desenvolvimento e são identificadas como demonstrativas.
- [x] Build de produção não inicia com dispositivos ou transferências mockados.
- [x] Home, Transferências, Configurações, sidebar e rota de dispositivo exibem estados derivados da fonte correta.
- [x] Testes cobrem bootstrap web, estado não configurado, fonte de fixtures, vazio e lifecycle sem listeners duplicados.

## Validação

- `npm run typecheck`
- `npm test`
- `npm run build`
- `npm run test:rust`
- `cargo check --manifest-path src-tauri/Cargo.toml`
- `git diff --check`
- Smoke test visual de Início, Transferências, Histórico, Configurações e abas de dispositivo em desktop, aproximadamente `680px` e `390px`.

### Execução realizada

- `npm run typecheck` — passou.
- `npm test` — passou: 6 arquivos, 25 testes.
- `npm run build` — passou; o build de produção não inclui fixtures como estado inicial.
- `npm run test:rust` — passou: 26 testes Rust, 0 falhas.
- `cargo check --manifest-path src-tauri/Cargo.toml` — passou.
- `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` — passou.
- `git diff --check` — passou.
- Smoke test web local passou nas rotas principais e nas larguras `1280`, `680` e `390`; não houve overflow horizontal além da barra de rolagem esperada nem erros de console.
