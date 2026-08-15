# TASK 06 — Preparar a base de testes e fixtures

Status: base mínima de testes, fixtures e comandos implementada; integrações de produto continuam futuras.

## Objetivo

Criar uma suíte local, determinística e pequena para validar invariantes do domínio, contratos da bridge, componentes Vue e cenários de peers/erros sem depender de dispositivos reais, de rede ou dos mocks usados pela apresentação.

Esta task prepara infraestrutura e casos mínimos. Ela não implementa runtime Rust, discovery, transporte, persistência, comandos Tauri de produto, stores reais ou integração com dispositivos.

## Estado atual

- Antes desta task, o `package.json` possuía somente scripts de desenvolvimento, build e typecheck, sem runner ou dependência de teste (`package.json:1-34` no estado inicial).
- O `tsconfig.json` inclui somente o código da aplicação e `vite.config.ts`; testes TypeScript ainda não participam do typecheck (`tsconfig.json:1-25`).
- Os tipos canônicos TypeScript já expõem estados, tabelas de transição, eventos e versão do modelo (`src/types/index.ts:26-85,391-530`), mas não têm fixtures versionadas nem validação de cenários.
- Os modelos puros Rust implementam `can_transition_to` e estados terminais, mas não têm módulos `#[cfg(test)]`, testes de integração ou relógio controlável (`src-tauri/src/domain/mod.rs:46-186,379-430,706-778`).
- `src/stores/devices.ts` e `src/stores/transfers.ts` usam arrays fixos de `MockDevice`/`MockTransfer`; esses dados são apresentação e não podem virar a fonte das fixtures de domínio (`src/stores/devices.ts:1-42`, `src/stores/transfers.ts:1-31`, `src/types/index.ts:520-548`).
- O contrato da TASK 05 exige versões, envelopes, eventos com sequência, erros fechados, fallback web honesto e ressincronização; ainda não há bridge implementada para ser usada nos testes (`docs/tasks/TASK-05-contrato-da-bridge-rust-vue.md:44-234`).
- A arquitetura e as tasks anteriores exigem distinguir presença, trust, capability, estado de operação, loading, stale, offline, recusa e conclusão, sem depender de um booleano ou de tempo relativo (`SYSTEM-DESIGN.md:68-93,138-146`, `docs/tasks/TASK-01-modelos-de-dominio-e-estados.md:95-111`).

## Brainstorm

### Alternativas consideradas

| Alternativa | Avaliação | Decisão |
| --- | --- | --- |
| Vitest integrado ao Vite | Usa a mesma transformação/alias do app, suporta TypeScript e `.vue`, tem modo watch/run e não exige trocar a configuração existente. | Escolhida |
| Jest + transformers Vue | Adiciona uma cadeia de transformação separada para Vite/Vue e aumenta a configuração antes de haver necessidade. | Rejeitada |
| Vitest em ambiente `happy-dom` para tudo | Simplifica a configuração, mas torna testes puros dependentes de DOM e esconde a separação entre domínio e UI. | Rejeitada como ambiente global |
| Node por padrão + `happy-dom` apenas nos testes Vue | Mantém testes de domínio/fixtures rápidos e usa DOM somente onde o componente precisa. | Escolhida |
| Vue Test Utils com snapshots amplos | Captura markup e classes frágeis, sem provar estados ou comportamento observável. | Rejeitada como estratégia principal |
| Vue Test Utils com asserções de saída e interação | É adequada ao Vue 3, testa o que aparece/acontece no componente e evita acoplamento a implementação. | Escolhida |
| `rstest`, `proptest` ou outro framework Rust | Pode ser útil para geração de casos depois, mas adiciona dependências antes de haver lógica complexa ou propriedades estabilizadas. | Adiada |
| `#[test]`/`cargo test` nativo | Já é fornecido pelo Rust, suficiente para transições e invariantes determinísticas da TASK 01. | Escolhida |
| Reutilizar arrays dos stores como fixtures | Mistura copy, tempo relativo e status de apresentação com o contrato canônico. | Rejeitada |
| Peer falso com socket/QUIC real | Anteciparia networking e tornaria a suíte dependente de porta, interface e firewall; pertence às tasks de discovery/protocolo. | Rejeitada nesta task |
| Peer falso como máquina determinística de estados/eventos | Exercita presença, trust, capability, sequência e falhas sem alegar comunicação real. | Escolhida |

### Perguntas que a base precisa responder

1. Um teste falho aponta para a fixture, para o contrato ou para a implementação que violou a expectativa?
2. Como avançar tempo sem esperar pelo relógio do sistema e sem usar textos como `agora`?
3. Como criar peers falsos que representem offline, stale, capability negada e eventos duplicados sem abrir sockets?
4. Como testar Rust, TypeScript e Vue com comandos claros em uma instalação limpa?
5. Como impedir que uma fixture de teste vire import acidental no app ou que um mock visual passe a representar produto?

## Decisões

### 1. Ferramentas e ambientes

- **TypeScript/Vue:** Vitest como runner, aproveitando Vite 7, ESM, alias `@/*` e transformação de `.vue` já usada pelo app.
- **Componentes Vue:** `@vue/test-utils` v2, que é a linha compatível com Vue 3, com asserções centradas em texto, atributos, eventos e interação.
- **DOM:** `happy-dom` somente nos arquivos que montam componentes; os testes de domínio, fixtures, bridge e peers usam o ambiente Node padrão.
- **Rust:** `cargo test --manifest-path src-tauri/Cargo.toml`, usando `#[test]` e `#[cfg(test)]`/testes de integração sem crate adicional.
- **Relógio:** helper próprio em `tests/support/test-clock.ts`, baseado em um instante inicial fixo e avanço explícito em milissegundos.

As escolhas seguem a documentação oficial: Vitest é um runner nativo do Vite e procura arquivos `.test`/`.spec`; Vue Test Utils v2 mira Vue 3 e recomenda Vitest; `happy-dom` é um ambiente opcional do Vitest para APIs de navegador; Rust fornece o runner `cargo test` ([Vitest](https://vitest.dev/guide/index.html), [ambientes Vitest](https://vitest.dev/guide/environment), [Vue Test Utils](https://test-utils.vuejs.org/guide/), [testes Rust](https://doc.rust-lang.org/book/ch11-01-writing-tests.html)).

### 2. Scripts e configuração

O `package.json` terá comandos explícitos:

```json
{
  "test": "vitest run",
  "test:watch": "vitest"
}
```

`vitest.config.ts` reutilizará a configuração Vite existente, preservando plugin Vue e alias. O ambiente padrão será `node`; o teste de componente declarará `// @vitest-environment happy-dom`. Os testes serão incluídos no `tsconfig.json` para que `npm run typecheck` também confira fixtures e specs.

Não haverá snapshot global, cobertura obrigatória ou browser real nesta task. A suíte deve ser rápida e determinística; cobertura e Browser Mode podem ser adicionados quando houver bridge e serviços reais para justificar o custo.

### 3. Organização e ownership

```text
tests/
  fixtures/
    domain.ts
    bridge.ts
  support/
    test-clock.ts
    fake-peer.ts
  domain-transitions.test.ts
  bridge-contract.test.ts
  fake-peer.test.ts
  ui-button.test.ts
src-tauri/tests/
  domain_transitions.rs
```

- `tests/fixtures/` contém dados canônicos de teste, versionados e sem copy de apresentação.
- `tests/support/` contém somente harnesses determinísticos; não é importado por `src/`.
- `tests/*.test.ts` cobre contratos TypeScript, peers falsos e smoke de Vue.
- `src-tauri/tests/domain_transitions.rs` cobre as mesmas invariantes essenciais no lado Rust sem editar o módulo de produção para inserir casos de teste.
- `src-tauri/src/domain/mod.rs` não será duplicado nem convertido em fixture; a integração testa a API pública do módulo.

Cada fixture exportará `FIXTURE_VERSION = 1`. Uma alteração incompatível no formato exige nova versão ou migração explícita do fixture; não haverá atualização silenciosa que faça um teste continuar passando com dados semanticamente diferentes.

### 4. Relógio controlável

O `TestClock` começa em um instante UTC fixo, expõe `now()`, `advance(ms)` e `set(ms)` e retorna `UtcTimestamp` compatível com o domínio. Nenhum teste dependerá de `Date.now()`, `setTimeout` real ou copy relativa. O relógio controla somente a fixture/harness; não altera o relógio do sistema nem a produção.

Casos mínimos:

- candidate dentro do prazo e expirado após avanço;
- presença `online → stale → offline` sem mudar trust;
- pairing com expiração sem virar `confirmed`;
- transferências com timestamps ordenados;
- evento duplicado ou fora de ordem identificado pela fixture de sequência.

### 5. Fixtures canônicas e peers falsos

As factories criarão objetos mínimos e completos para `Device`, `Presence`, `PairingSession`, `CapabilityGrant`, `TransferSession`, `DomainEvent`, `BridgeEvent` e `BridgeError`, com IDs explícitos, timestamps do relógio e `DOMAIN_MODEL_VERSION`/`bridgeContractVersion` fixos.

O `FakePeer` será uma máquina local sem sockets com:

- identidade/ID e metadados apresentados;
- presença controlável (`unknown`, `online`, `stale`, `offline`);
- trust e capabilities separados;
- fila de eventos com `streamId`, `sequence` e `eventId`;
- ações de duplicar, atrasar, perder, rejeitar e encerrar evento;
- erro público redigido, sem path, token, endpoint ou payload sensível.

O peer falso não representa discovery, QUIC, pairing criptográfico ou transporte funcional. Ele fornece entradas determinísticas para as tasks que implementarão essas camadas.

### 6. Casos negativos obrigatórios

As fixtures e testes devem cobrir:

- transição inválida e transição terminal;
- presença stale/offline sem revogar trust;
- capability em direção errada, negada ou revogada;
- pairing expirado/rejeitado sem conceder trust;
- transfer não concluída sem `result` confirmado;
- bridge com `bridgeContractVersion`/`modelVersion` incompatível;
- erro não retryable versus retryable;
- evento duplicado, gap de sequência e payload inválido;
- peer offline, evento perdido e retorno posterior online;
- conteúdo de fixture sem chave privada, token, path completo, SQL ou payload remoto.

Falhas de fixture devem usar nomes e mensagens que apontem para a expectativa quebrada. A suíte não deve aceitar `any`, `as unknown as` indiscriminado ou cast que esconda um campo obrigatório.

## Plano de implementação

1. Instalar `vitest`, `@vue/test-utils` e `happy-dom` como dev dependencies e registrar versões no lockfile.
2. Criar `vitest.config.ts`, incluir testes no `tsconfig.json` e adicionar scripts `test`/`test:watch`.
3. Criar o `TestClock`, factories versionadas, tipos de envelope de fixture e `FakePeer` sem rede.
4. Adicionar testes TypeScript para transições, versão/forma de fixtures, erros, sequência de eventos e relógio.
5. Adicionar teste Vue isolado para renderização/interação observável de `Button.vue`, usando `happy-dom` e sem importar stores mockados.
6. Adicionar teste Rust de integração para transições de presença, pairing, trust, capability, transferência, notification e comando remoto.
7. Rodar a suíte em modo isolado, provocar uma falha controlada de fixture para confirmar diagnóstico e remover a alteração temporária antes da entrega.
8. Atualizar `TODO.md` e `SYSTEM-DESIGN.md` somente depois de a suíte base passar e o diff ser revisado.

## Execução paralela

A investigação foi separada em recortes sem escrita sobreposta:

- **Ferramentas:** conferência da configuração Vite/TypeScript, versões Node/npm/Vite e documentação oficial de Vitest, Vue Test Utils e Rust.
- **Contratos e cenários:** auditoria dos tipos/transições TypeScript/Rust, TASKS 01/03/04/05 e separação entre fixtures canônicas e mocks visuais.

A implementação será sequenciada por ownership: configuração/dependências, harness/fixtures, testes TypeScript/Vue e teste Rust. Não haverá edição paralela dos arquivos compartilhados `package.json`, `package-lock.json`, `tsconfig.json` ou `TODO.md`.

## Integração

- A TASK 07 deve usar o relógio e os erros públicos definidos aqui nos testes de inicialização parcial, sem introduzir um relógio global implícito.
- A TASK 08 deve acrescentar fixtures de migração em diretório próprio, sem reutilizar o estado dos stores.
- A TASK 09 deve reutilizar os casos de versão, envelopes, erros, eventos duplicados/gaps e listener lifecycle ao implementar a bridge.
- A TASK 10 deve adicionar testes de stores/adaptadores para snapshot, stale/offline, respostas atrasadas e eventos duplicados sem remover a fronteira dos fixtures.
- As TASKS 11–22 poderão trocar `FakePeer` por peers de protocolo próprios quando houver implementação de rede; os testes desta task continuarão sendo unitários e offline.
- Os mocks em `src/stores/`, `src/types/index.ts` e componentes de apresentação permanecem fora da base canônica de fixtures.
- Nenhum teste deve exigir dispositivo real, interface LAN, daemon Avahi, keyring, banco local, diretório do usuário ou credencial.

## Critérios de conclusão

- [x] A suíte TypeScript/Vue executa em instalação limpa com `npm test`.
- [x] O `cargo test --manifest-path src-tauri/Cargo.toml` executa invariantes do domínio sem rede ou persistência.
- [x] Vitest, Vue Test Utils e `happy-dom` estão configurados e documentados; Node é usado por padrão e DOM somente quando necessário.
- [x] Fixtures canônicas versionadas estão separadas dos mocks visuais e não são importadas pela aplicação.
- [x] Há relógio controlável e testes que verificam expiração/timeout sem esperar tempo real.
- [x] Há peer falso determinístico com presença, trust, capability, eventos, duplicação, atraso, perda e erro.
- [x] Há casos positivos e negativos de transições, bridge, versão, eventos, erros e UI.
- [x] Uma falha de fixture produz diagnóstico localizável; nenhum caso depende de dispositivo, rede, segredo ou filesystem do usuário.
- [x] O typecheck, build e smoke test `greet` continuam passando.
- [x] A documentação marca a base de testes como implementada e mantém as integrações reais como futuras.

## Validação

### Evidência revisada

- `package.json:6-34` e `tsconfig.json:1-25` registram a configuração inicial sem runner; a implementação desta task adicionou os scripts, dependências e specs descritos abaixo.
- `src/types/index.ts:26-85,391-530` e `src-tauri/src/domain/mod.rs:46-186,379-430,706-778` fornecem estados, transições, terminais e versão que a base deve exercitar.
- `src/stores/devices.ts:1-42`, `src/stores/transfers.ts:1-31` e `src/types/index.ts:520-548` confirmam que mocks visuais não são fixtures canônicas.
- `docs/tasks/TASK-05-contrato-da-bridge-rust-vue.md:44-234` define o envelope, versões, eventos, erros, gaps, prévia web e dados proibidos que os testes devem representar.
- A documentação oficial consultada confirma Vitest sobre Vite, ambientes Node/`happy-dom`, Vue Test Utils v2 para Vue 3 e o runner nativo `cargo test`.
- `npm install --save-dev vitest @vue/test-utils happy-dom` registrou as dependências no `package.json`/`package-lock.json`; a instalação terminou sem vulnerabilidades reportadas.

### Matriz mínima de execução

| Comando | Cobertura | Ambiente |
| --- | --- | --- |
| `npm test` | fixtures, domínio/contratos TS, fake peers e componente Vue | Node + `happy-dom` pontual |
| `npm run typecheck` | aplicação, configuração e specs TypeScript | TypeScript estrito |
| `cargo test --manifest-path src-tauri/Cargo.toml` | invariantes e transições Rust | offline, sem Tauri runtime |
| `npm run build` | compatibilidade do app sem imports de teste na produção | Vite |

### Falha controlada de fixture

Uma expectativa de `bridgeContractVersion` foi alterada temporariamente para `999`. O teste falhou com `expected 1 to be 999` em `tests/bridge-contract.test.ts:14`, apontando fixture, campo e expectativa; a linha foi restaurada e a suíte voltou a passar.

### Execução realizada

- `npm test`: 4 arquivos e 14 testes TypeScript/Vue aprovados.
- `npm run typecheck`: aprovado incluindo `tests/**/*.ts` e `vitest.config.ts`.
- `npm run test:rust`: 4 testes Rust de integração aprovados, além de unit/doc tests sem casos.
- `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`: aprovado.
- `npm run build`: aprovado; os testes não entram no bundle de produção.
