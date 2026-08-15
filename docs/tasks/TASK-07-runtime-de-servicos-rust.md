# TASK 07 — Estruturar o runtime de serviços Rust

Status: implementação concluída; networking, persistência, bridge de produto e efeitos locais continuam futuros

## Objetivo

Criar o ciclo de vida interno dos serviços do Pulse no processo Tauri, com estado compartilhado, ordem determinística de inicialização, encerramento reverso, falha parcial observável e fronteiras claras entre o domínio, os efeitos e a bridge.

Esta task prepara a orquestração. Ela não implementa discovery, pairing, identidade, SQLite, keyring, transporte, comandos de produto, eventos IPC ou qualquer efeito local.

## Estado atual

- `src-tauri/src/lib.rs:1-20` expõe somente o módulo puro `domain`, registra o command `greet` e inicia um `tauri::Builder` sem estado gerenciado ou ciclo de vida de serviços.
- `src-tauri/src/main.rs:1-3` delega diretamente para `pulse_lib::run()`; não há ponto explícito para start/stop dos serviços.
- `src-tauri/Cargo.toml:1-19` contém somente Tauri e `tauri-build`; não há crate de persistência, rede, serialização, keyring ou runtime assíncrono adicionada para esta task.
- `SYSTEM-DESIGN.md:78-104,123-146` confirma que a bridge de produto, os serviços e os efeitos ainda não existem; os diretórios de domínio são apenas pontos de organização.
- A TASK 04 exige que storage seja dependência explícita, falhe de modo recuperável e não exponha SQL, paths ou corrupção à UI (`docs/tasks/TASK-04-persistencia-migracoes-e-retencao-local.md:91-102,128-131`).
- A TASK 05 exige que a bridge consiga reportar runtime parcial ou não configurado sem fabricar serviços disponíveis (`docs/tasks/TASK-05-contrato-da-bridge-rust-vue.md:240-258`).
- A TASK 06 já fornece testes Rust offline e pede que a TASK 07 cubra inicialização parcial sem relógio global implícito (`docs/tasks/TASK-06-base-de-testes-e-fixtures.md:160-170,200-214`).
- O Tauri 2.11.5 usado no ambiente executa o hook `setup` durante o build da aplicação e expõe `RunEvent::Exit` no callback de `App::run`; isso permite ligar start e shutdown sem armazenar `AppHandle` no domínio (`/home/kyle/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tauri-2.11.5/src/app.rs:66-70,220-248,1345-1380`).

## Brainstorm

### Alternativas de orquestração

| Alternativa | Avaliação | Decisão |
| --- | --- | --- |
| Estado global mutável acessado por commands | Simplifica o primeiro command, mas mistura lifecycle, bridge e domínio e torna testes de inicialização difíceis. | Rejeitada |
| `AppHandle` guardado em cada serviço | Facilita emitir eventos cedo, mas acopla serviços ao Tauri e permite que efeitos internos atravessem a fronteira sem contrato. | Rejeitada |
| `Runtime` puro com `RuntimeState` fino para Tauri | Permite testar ordem, falha e encerramento sem criar uma aplicação Tauri; a bridge recebe o estado somente nas tasks futuras. | Escolhida |
| Inicializar todos os módulos como se fossem ativos | Criaria uma falsa impressão de networking/persistência pronta. | Rejeitada |
| Registro ordenado por enum e dependências implícitas | Mantém uma ordem determinística e torna a precedência revisável antes de adicionar serviços reais. | Escolhida |
| Runtime assíncrono e tasks de fundo nesta etapa | Adiciona executor, cancelamento e shutdown concorrente antes de existirem serviços que precisem disso. | Adiada |

### Perguntas que a implementação precisa responder

1. Como distinguir serviço não configurado de serviço deliberadamente inativo e de serviço que falhou?
2. Em que ordem os serviços começam, e como os já iniciados são limpos se o próximo falhar?
3. Como o runtime encerra em ordem reversa sem depender de callbacks da UI ou de uma janela Tauri?
4. Como expor erro suficiente para diagnóstico sem vazar SQL, paths, sockets, tokens, payloads ou segredos?
5. Como manter `greet` e a prévia web funcionando sem transformar o estado interno em API de produto?

## Decisões

### 1. Runtime puro e estado compartilhado

- `Runtime` será um coordenador Rust puro, sem `tauri::AppHandle`, `WebviewWindow`, command, evento, socket ou conexão de armazenamento.
- `RuntimeState` será o único wrapper compartilhável gerenciado pelo Tauri, usando `Arc<Mutex<Runtime>>`. Ele oferece apenas operações internas de `start`, `shutdown` e leitura de snapshot para a própria aplicação e para testes.
- O snapshot não será DTO da bridge nesta task. Não terá `Serialize`, `Deserialize`, `serde_json`, conteúdo de domínio ou detalhes de implementação; a conversão pública pertence à TASK 09.
- O runtime será criado antes do `setup`, iniciado pelo hook de setup e encerrado quando o callback de `RunEvent::Exit` for recebido.

### 2. Catálogo e estados de serviço

O catálogo inicial representa fronteiras futuras, não implementações presentes:

`Storage`, `Identity`, `DeviceRegistry`, `Discovery`, `Pairing`, `Protocol`, `Transfer`, `Clipboard`, `Media` e `Notifications`.

Cada slot terá um estado observável internamente:

- `not-configured`: não há implementação registrada; não pode ser usado como serviço disponível;
- `inactive`: o serviço foi reconhecido, mas está desabilitado nesta configuração;
- `stopped`: serviço configurado, porém ainda não iniciado ou já encerrado;
- `starting`, `running`, `stopping` e `failed`: estados transitórios/terminais do lifecycle.

O runtime terá fases `created`, `starting`, `partial`, `ready`, `failed`, `stopping` e `stopped`. `partial` será usado quando houver serviços ausentes/inativos, mesmo que os serviços configurados tenham iniciado; `ready` só ocorrerá quando todos os slots do catálogo estiverem configurados e rodando. Assim, o estado padrão nunca sugere que discovery, persistência ou transporte estão ativos.

### 3. Ordem e dependências

A ordem inicial será fixa e explícita:

`Storage → Identity → DeviceRegistry → Discovery → Pairing → Protocol → Transfer → Clipboard → Media → Notifications`.

O encerramento percorre a ordem inversa. A lista é uma fronteira de orquestração, não uma autorização para implementar os módulos. Dependências futuras devem continuar sendo expressas pelo registro/lifecycle do serviço, sem chamar diretamente outro módulo por dentro do bridge.

### 4. Contrato mínimo de serviço

Um serviço registrado implementará uma trait interna com `kind`, `start` e `stop`. Os métodos retornam somente códigos fechados de falha (`initialization-failed`, `dependency-unavailable`, `shutdown-failed`); nenhum erro bruto é guardado ou serializado pelo runtime.

O método `stop` deverá ser seguro após uma inicialização parcial e será chamado em ordem reversa para todo serviço que possa ter adquirido recursos. Serviços ausentes não são substituídos por no-ops que aleguem disponibilidade.

### 5. Falhas e cleanup

- O primeiro erro de start impede que serviços posteriores sejam iniciados.
- O runtime tenta limpar o serviço que falhou e todos os serviços iniciados anteriormente, em ordem reversa, continuando mesmo se um cleanup falhar.
- A chamada retorna um erro estruturado com serviço, etapa e código fechado; o snapshot fica em `failed` e preserva quais slots foram limpos ou permaneceram falhos.
- No shutdown normal, todos os serviços running são parados em ordem reversa. Falhas de stop não são engolidas: os demais serviços continuam sendo encerrados, o runtime fica `failed` e o erro é retornado.
- `start` não pode ser chamado duas vezes no mesmo runtime. `shutdown` é idempotente depois de `stopped` e pode repetir cleanup de slots `failed`.

### 6. Integração Tauri mínima

- `lib.rs` gerenciará uma instância de `RuntimeState`, iniciará o runtime no `setup` e manterá `greet` no `invoke_handler` sem alterações de contrato.
- O callback de `RunEvent::Exit` chamará `shutdown`. Se o encerramento falhar, somente um diagnóstico fechado será escrito no stderr; nenhum detalhe de serviço interno ou dado sensível será exposto.
- Nenhum command novo será registrado, nenhuma capability Tauri será adicionada e a prévia web não será alterada.

## Plano de implementação

1. Criar `src-tauri/src/runtime/mod.rs` com catálogo, fases, estados, códigos de falha, trait de serviço, registro ordenado, `Runtime`, `RuntimeState` e snapshots internos.
2. Implementar transições de lifecycle, cleanup reverso, idempotência de shutdown e tratamento de mutex envenenado sem panics de domínio.
3. Integrar `RuntimeState` ao `tauri::Builder` no `setup` e no callback de `RunEvent::Exit`, preservando `greet` como smoke test.
4. Adicionar testes Rust de integração com serviços falsos locais para ordem de start/stop, runtime padrão parcial, serviço inativo, falha de start, falha de cleanup, falha de shutdown e transições inválidas.
5. Atualizar `SYSTEM-DESIGN.md`, `README.md` e `TODO.md` somente após revisar o diff e confirmar que o runtime não declara funcionalidades de produto como implementadas.

## Execução paralela

A investigação foi separada em dois recortes sem escrita sobreposta:

- **Arquitetura e contratos:** cruzamento das TASKS 03–06, `SYSTEM-DESIGN.md`, `TODO.md` e os modelos atuais para delimitar serviços, estados, dados proibidos e integração futura da bridge.
- **Lifecycle Tauri:** inspeção do código-fonte da versão instalada do Tauri para confirmar o tipo do hook `setup`, o registro de estado e o callback `RunEvent::Exit`.

A implementação será sequenciada porque `lib.rs`, `runtime/mod.rs`, os testes e a documentação compartilham a mesma decisão de lifecycle. Não há paralelismo real adicional que justifique editar esses arquivos em paralelo.

## Integração

- A TASK 08 registrará o storage real no slot `Storage`; falha de migration deverá usar os mesmos códigos fechados e impedir serviços dependentes de iniciar.
- A TASK 09 poderá adaptar snapshots e erros para envelopes públicos, sem expor a trait, `Mutex`, slot, `AppHandle` ou detalhes de cleanup.
- As TASKS 11–22 registrarão discovery, identidade, pairing e protocolo apenas quando suas implementações existirem; `not-configured` continuará sendo o estado honesto enquanto isso não ocorrer.
- As tasks de recursos registrarão Transfer, Clipboard, Media e Notifications sob as mesmas fronteiras; nenhuma implementação deverá acessar a UI diretamente.
- Os testes permanecerão offline, sem sockets, banco, keyring, filesystem do usuário, janela Tauri ou credenciais.

## Critérios de conclusão

- [x] Existe um `Runtime` puro e testável, separado de Tauri, domínio e efeitos.
- [x] O Tauri gerencia estado compartilhado, inicia no `setup` e solicita shutdown no evento de saída.
- [x] A ordem de start é determinística e o shutdown percorre a ordem inversa.
- [x] Serviços não configurados e inativos aparecem como tais e não são contados como disponíveis.
- [x] Falhas de inicialização e encerramento retornam códigos estruturados sem dados sensíveis.
- [x] Inicialização parcial limpa recursos adquiridos e preserva o diagnóstico de slots que falharam.
- [x] `greet`, mocks, prévia web e capabilities Tauri atuais continuam inalterados em comportamento.
- [x] Há testes Rust para sucesso parcial, ordem, falha, cleanup, idempotência e transições inválidas.
- [x] `npm run typecheck`, `npm test`, `npm run test:rust`, `npm run build`, `cargo check` e `cargo fmt --check` passam.

## Validação

### Evidências a confirmar após implementação

- `src-tauri/src/runtime/mod.rs` concentra o lifecycle sem importar `tauri` ou os módulos de efeitos.
- `src-tauri/src/lib.rs` registra o estado e os hooks de lifecycle; `greet` permanece o smoke test e os commands tipados da bridge são adicionados pela TASK 09.
- `src-tauri/tests/runtime_lifecycle.rs` não abre rede, banco, keyring ou diretórios do usuário.
- `SYSTEM-DESIGN.md` e `TODO.md` distinguem runtime estruturado de serviços de produto implementados.

### Matriz mínima

| Cenário | Resultado exigido |
| --- | --- |
| Runtime padrão inicia | Fase `partial`; todos os serviços continuam `not-configured`. |
| Serviço explicitamente inativo | Fica `inactive`; não recebe `start` nem aparece como disponível. |
| Serviços configurados iniciam | Start segue a ordem do catálogo; snapshot mostra `partial` se ainda houver slots ausentes. |
| Serviço falha no start | Serviços posteriores não iniciam; cleanup reverso ocorre e o erro identifica etapa/código. |
| Stop de um serviço falha | Demais serviços são tentados; runtime termina `failed` e retorna erro fechado. |
| Shutdown repetido | Depois de `stopped`, não chama serviços novamente e retorna snapshot estável. |
| Tauri inicia/encerra | `setup` e `RunEvent::Exit` usam o estado gerenciado sem mudar `greet` ou adicionar command. |

### Execução realizada

- `cargo test --manifest-path src-tauri/Cargo.toml`: 11 testes aprovados, incluindo os 4 testes de domínio existentes e 7 testes do runtime.
- `cargo check --manifest-path src-tauri/Cargo.toml`: aprovado.
- `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`: aprovado.
- `npm test`: 4 arquivos e 14 testes TypeScript/Vue aprovados.
- `npm run typecheck`: aprovado.
- `npm run build`: aprovado; o runtime Rust não altera o bundle web.
- `git diff --check`: aprovado.

Os critérios acima foram atendidos sem adicionar dependências de rede, persistência, keyring, serialização ou capabilities Tauri. O runtime padrão permanece `partial` com todos os serviços de produto `not-configured`; a TASK 08 é o próximo passo recomendado.
