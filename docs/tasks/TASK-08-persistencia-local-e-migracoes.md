# TASK 08 — Implementar persistência local e migrações

Status: implementação concluída; hidratação da UI, identidade/keyring, bridge de produto e recursos continuam futuros

## Objetivo

Tirar o primeiro conjunto de metadados confiáveis da memória sem criar dependência de cloud, mantendo o banco exclusivamente atrás do runtime Rust e preservando a política da TASK 04 para segredos, Clipboard, conteúdo leve, caminhos, tokens e payloads.

Esta task implementa a fundação de armazenamento: abertura em `appLocalDataDir`, configuração segura da conexão SQLite, migrations versionadas e verificadas por checksum, schema inicial, APIs Rust tipadas para metadados não sensíveis e testes offline de instalação, upgrade, incompatibilidade, rollback, reinício e limpeza. Ela não implementa hidratação dos stores Vue, identidade privada no Secret Service, pairing, trust/capabilities de produto, histórico funcional, transferências ou bridge de produto.

## Estado atual

- `TODO.md:85-93` identifica a TASK 08 como a próxima pendência da Fase 2 e exige abertura, migrações, preservação de dados e falha recuperável.
- A TASK 04 escolheu SQLite local atrás de um adaptador Rust, determinou `appLocalDataDir`, forward-only migrations, checksum, foreign keys, journaling, recuperação sem recriação silenciosa e a separação entre banco, keyring e memória (`docs/tasks/TASK-04-persistencia-migracoes-e-retencao-local.md:39-111`).
- O runtime agora possui o slot `Storage`, ordem `Storage → Identity → ...`, falhas com códigos fechados e cleanup reverso (`src-tauri/src/runtime/mod.rs:1-170,178-400`; `docs/tasks/TASK-07-runtime-de-servicos-rust.md:47-91`). O slot ainda está `not-configured` no runtime padrão.
- O Tauri já usa o identificador `com.pulse.desktop` (`src-tauri/tauri.conf.json:3-6`) e a aplicação registra o runtime no `setup` (`src-tauri/src/lib.rs:13-31`), mas não abre banco nem executa migration.
- Os modelos puros têm IDs, trust, capabilities, sessões de pairing/transferência, histórico e notificações, mas não possuem serialização ou repositório (`src-tauri/src/domain/mod.rs:1-20,245-350,443-535`; `src/types/index.ts:91-173,258-335,391-525`).
- Os testes Rust existentes são offline e não podem acessar o diretório do usuário nem reutilizar os mocks de apresentação (`docs/tasks/TASK-06-base-de-testes-e-fixtures.md:74-98,164-183`; `src-tauri/tests/runtime_lifecycle.rs`).

## Brainstorm

### Alternativas consideradas

| Alternativa | Avaliação | Decisão |
| --- | --- | --- |
| `rusqlite` com SQLite embutido | API síncrona adequada ao lifecycle atual do Tauri, controle explícito de pragmas/transações e sem SQL exposto à UI; `bundled` reduz dependência de biblioteca do sistema. | Escolhida |
| `sqlx` com runtime assíncrono | Bom para aplicações assíncronas, mas adiciona runtime, pool e complexidade antes de haver serviços concorrentes; não é necessário para a conexão única desta task. | Adiada |
| `tauri-plugin-sql` | Oferece comandos SQL genéricos e migrations acessíveis pela bridge, contrariando a fronteira tipada da TASK 04/05. | Rejeitada |
| `rusqlite_migration` | Poderia aplicar migrations básicas, mas o Pulse precisa registrar nome, checksum e política de versão futura de forma explícita; o runner local fica pequeno e auditável. | Rejeitada nesta etapa |
| JSON/TOML ou vários arquivos | Exige implementar locking, atomicidade, relações, checksum e recuperação por conta própria. | Rejeitada |
| SQLite em WAL com `synchronous=FULL` | Permite leitura concorrente futura e mantém durabilidade forte; os arquivos auxiliares são tratados como parte do banco em diagnóstico/reset. | Escolhida |
| Rollback/downgrade automático | Pode destruir dados válidos e não tem semântica segura para builds antigos. | Rejeitada |

### Perguntas que a implementação precisa responder

1. Uma instalação nova cria o diretório, abre o banco, aplica o schema completo e deixa a conexão configurada?
2. Um banco já migrado aplica somente versões posteriores e rejeita checksum alterado ou versão futura?
3. Uma migration que falha deixa o banco anterior abrível, sem apagar ou recriar dados?
4. Corrupção, foreign key quebrada, lock e erro de escrita retornam estados redigidos e recuperáveis?
5. As APIs de repositório conseguem persistir metadados sem aceitar conteúdo de Clipboard, URL, path completo ou segredo?
6. Reiniciar a abertura preserva registros e a limpeza explícita remove somente dados elegíveis?

## Decisões

### 1. Crate e configuração da conexão

- Usar `rusqlite` `0.40.2` com feature `bundled`, fixando a implementação SQLite junto ao build Rust e evitando depender de uma versão disponível no host Linux.
- Usar `sha2` `0.10` somente para calcular o checksum hexadecimal das migrations versionadas no repositório. O checksum é integridade de schema, não mecanismo de segredo.
- O arquivo de produto será `pulse.sqlite` em `app.path().app_local_data_dir()`, criado pelo Tauri e nunca montado a partir de `$HOME`, diretório do executável ou input da UI.
- A conexão habilita `foreign_keys=ON`, `journal_mode=WAL`, `synchronous=FULL` e `busy_timeout=5000`. `journal_mode=OFF` e `synchronous=OFF` não são aceitos pela configuração.
- A abertura valida `application_id` (`PULS`) e `user_version` como marcadores auxiliares; a fonte de compatibilidade e checksum será `schema_migrations`.
- `Storage` é síncrono e não implementa `Send`/`Sync` artificialmente. O runtime é o proprietário da conexão; futuras leituras concorrentes deverão introduzir uma decisão própria de pool ou worker.

As escolhas de transação e pragmas seguem a documentação primária: SQLite descreve transações como atômicas e recuperáveis, exige que `foreign_keys` seja habilitado fora de uma transação, define `quick_check`/`integrity_check` e documenta a relação entre WAL, `synchronous=FULL` e durabilidade. A API de `rusqlite` fornece `pragma_update`, transações com rollback no drop e `transaction_with_behavior` ([SQLite transactions](https://www.sqlite.org/lang_transaction.html), [SQLite pragmas](https://www.sqlite.org/pragma.html), [SQLite transactional](https://sqlite.org/transactional.html), [rusqlite Connection](https://docs.rs/rusqlite/latest/rusqlite/struct.Connection.html)).

### 2. Migrations forward-only

- A versão suportada inicial será `1`, com uma migration `001_initial_domain_storage`.
- Cada migration terá `version`, `name` e SQL imutável no código. O checksum SHA-256 do SQL é gravado em `schema_migrations` junto com `applied_at` e `result=applied`.
- A migration roda em transação `IMMEDIATE`; a linha de controle só é inserida antes do commit da mesma transação.
- A tabela `schema_migrations` será criada pela própria migration inicial. Em instalações novas, a ausência da tabela equivale à versão `0`.
- Ao reabrir, todas as migrations aplicadas são conferidas contra nome/checksum. Divergência retorna erro de integridade de migration e não tenta reparar automaticamente.
- Se a versão máxima aplicada for maior que `1`, a abertura retorna `Incompatible` sem escrever, fazer downgrade, apagar tabelas ou recriar o banco.
- Uma migration com falha retorna `MigrationFailed { version }`; o arquivo original permanece no lugar e a transação é descartada.

### 3. Schema inicial e fronteira de dados

A migration cria as tabelas definidas pela TASK 04: `schema_migrations`, `local_identity_public`, `known_devices`, `trust_relationships`, `capability_grants`, `revocation_blocks`, `pairing_sessions`, `transfer_sessions`, `history_entries`, `notification_records` e `preferences`, com foreign keys, checks de estado e índices mínimos. O schema armazena somente metadados não sensíveis.

- Não haverá coluna para chave privada, segredo de recuperação, token, nonce, transcript, assinatura, endpoint/IP/porta, payload de protocolo, Clipboard, URL/texto ou path completo.
- `known_devices.last_seen_at` é observação histórica; presença continua em memória e nunca é reidratada como `online`.
- `transfer_sessions` guarda contagens, estado, tentativa, erro e integridade; não guarda `LocalPath`, conteúdo nem token de retomada.
- `history_entries` guarda tipo, origem/destino, resultado, IDs relacionados, timestamps e motivo; não guarda payload.
- Repositórios tipados recebem estruturas de metadados próprias do storage. A bridge e os modelos de domínio não ganham `Serialize`/`Deserialize` ou acesso SQL nesta task.

### 4. Estados de erro e recuperação

`StorageError` terá variantes fechadas (`Database`, `Corrupt`, `Incompatible`, `MigrationFailed`, `MigrationChecksumMismatch`, `ForeignKeyViolation`, `Io`, `InvalidInput`), sem guardar ou imprimir a mensagem crua do SQLite, path, SQL, conteúdo ou segredo. O `StorageService` converte qualquer falha de abertura/migration em `InitializationFailed` no runtime, preservando o contrato fechado da TASK 07.

Na abertura, o storage verifica a aplicação, migrations, `quick_check` e foreign keys. Um banco ilegível/corrompido não vira banco vazio. A task não fará quarentena destrutiva nem reset automático: o arquivo permanece preservado para uma recuperação futura explícita. O reset/remoção de dados de produto permanece uma API futura que deverá fechar o banco antes de lidar com o arquivo e seus auxiliares.

### 5. Serviço no runtime

- `StorageService::new(path)` será registrado no slot `Storage` durante o `setup`, usando `app_local_data_dir()`.
- `RuntimeState` ganhará uma configuração única antes do start para trocar o runtime padrão não configurado pelo builder que contém o storage.
- O `start` abre e migra a conexão; o `stop` solta a conexão. Nenhum command novo, evento, capability Tauri ou import Vue será criado.
- Se o storage falhar, o Tauri não inicia os serviços dependentes e o runtime retorna somente o erro fechado já definido.

## Plano de implementação

1. Adicionar `rusqlite` com `bundled` e `sha2` ao `src-tauri/Cargo.toml`/lockfile.
2. Criar `src-tauri/src/storage/` com erros redigidos, checksum, migration runner, schema inicial, abertura/configuração e APIs tipadas mínimas.
3. Implementar `StorageService` para o slot `Storage` sem misturar conexão SQLite com a bridge ou o domínio puro.
4. Ajustar `RuntimeState`/`lib.rs` para configurar o storage no `setup` via `app_local_data_dir`, preservando `greet`.
5. Adicionar testes Rust offline para instalação nova, reapertura, upgrade com migration de teste, checksum divergente, versão futura, rollback, integridade/foreign key e limpeza de dados elegíveis.
6. Atualizar `SYSTEM-DESIGN.md`, `PRODUCT.md`, `README.md` e `TODO.md` para distinguir storage implementado de hidratação/recursos ainda planejados.
7. Rodar format, testes Rust, typecheck, testes TypeScript, build, check e revisar `git diff`/`git diff --check`.

## Execução paralela

A investigação foi separada em dois recortes sem escrita sobreposta:

- **Contrato e segurança:** auditoria da TASK 04, TASK 03, modelos Rust/TypeScript e runtime para definir schema, dados proibidos, erros e lifecycle.
- **SQLite e plataforma:** consulta à documentação oficial de SQLite/rusqlite, inspeção da API Tauri `app_local_data_dir` e compatibilidade do toolchain Rust para escolher crate, pragmas e transações.

A implementação será sequenciada porque `Cargo.toml`, migrations, runtime e documentação compartilham a mesma versão de schema. Não há paralelismo real adicional que justifique editar esses arquivos em paralelo.

## Integração

- A TASK 04 continua sendo a fonte da política de retenção e não retenção; este código não adiciona keyring nem persiste conteúdo.
- A TASK 06 fornece o padrão de testes offline; fixtures de migration ficam no teste Rust e não são importadas pela UI.
- A TASK 07 recebe o `StorageService` no slot existente, usando `InitializationFailed` sem vazar erro bruto.
- A TASK 09 poderá converter estados/errors do storage para o contrato da bridge, mas não receberá conexão, SQL ou path.
- A TASK 10 ainda deverá manter stores mockados até existir um adaptador de bridge; abrir o SQLite não torna a UI persistida.
- As TASKS 13, 17, 18, 24, 27, 30 e 33 usarão as tabelas e repositórios conforme seus próprios contratos, sem ampliar o schema com payload genérico.

## Critérios de conclusão

- [x] `rusqlite`/SQLite embutido e checksum de migration estão registrados no projeto.
- [x] Primeira abertura em `appLocalDataDir` cria diretório, banco e schema completo.
- [x] Pragmas de foreign keys, journaling, sincronização e contenção são aplicados e verificados.
- [x] Migrations são forward-only, versionadas, checksumadas, transacionais e rejeitam versão futura/alteração de migration aplicada.
- [x] O schema cobre os registros definidos na TASK 04 sem guardar segredo, payload, Clipboard, URL, path completo ou token.
- [x] Reabertura preserva dados válidos e não marca presença como online.
- [x] Falha de migration, corrupção, foreign key, banco incompatível e escrita não apaga nem recria o banco silenciosamente.
- [x] Storage está registrado no runtime Tauri como serviço real, sem comandos de produto novos.
- [x] Há testes offline de instalação, upgrade/runner, rollback, incompatibilidade, reinício, integridade e limpeza explícita.
- [x] `greet`, mocks, preview web, capabilities atuais e build frontend continuam válidos.

## Validação

### Evidência local revisada

- `TODO.md:85-93,409-411` — escopo da TASK 08 e próxima task recomendada.
- `docs/tasks/TASK-04-persistencia-migracoes-e-retencao-local.md:39-111,154-197` — backend, schema lógico, dados proibidos, migrations, corrupção, reset e cenários exigidos.
- `docs/tasks/TASK-06-base-de-testes-e-fixtures.md:74-98,164-183` — separação de fixtures e restrição a testes offline.
- `docs/tasks/TASK-07-runtime-de-servicos-rust.md:47-116` — slot `Storage`, ordem, códigos fechados e integração mínima Tauri.
- `src-tauri/src/runtime/mod.rs:1-170,178-455` — catálogo, lifecycle, configuração única, `RuntimeState` e transições existentes.
- `src-tauri/src/lib.rs:1-50` e `src-tauri/tauri.conf.json:3-6` — ponto de integração e identificador do app.
- `src-tauri/src/domain/mod.rs:1-20,245-350,443-535` — entidades e campos que não devem ser serializados diretamente para o banco.
- `src-tauri/src/storage/mod.rs:1-21,23-67,69-252,284-613,616-840` — crate, schema, erros redigidos, abertura, migrations, APIs internas e serviço do runtime.
- `src-tauri/tests/storage.rs:1-274` — testes de instalação, reinício, schema futuro, checksum, foreign keys, corrupção, limpeza e lifecycle.

### Fontes primárias consultadas

- [SQLite transactions](https://www.sqlite.org/lang_transaction.html) — transações, rollback e contenção de escrita.
- [SQLite pragmas](https://www.sqlite.org/pragma.html) — `foreign_keys`, `journal_mode`, `synchronous`, `quick_check` e `integrity_check`.
- [SQLite transactional](https://sqlite.org/transactional.html) — atomicidade, consistência, isolamento e durabilidade.
- [rusqlite Connection](https://docs.rs/rusqlite/latest/rusqlite/struct.Connection.html) — `pragma_update`, transações e rollback no drop.

### Matriz de validação

| Cenário | Resultado exigido |
| --- | --- |
| Primeira abertura | Diretório/banco são criados e a versão suportada fica registrada uma vez. |
| Reabertura | Registros válidos permanecem, sem reaplicar migration nem marcar presença online. |
| Migration de upgrade | Somente a versão posterior é aplicada; dados anteriores permanecem. |
| Migration interrompida | A transação reverte e o banco anterior continua abrível. |
| Checksum alterado | A abertura falha como incompatível/integridade, sem escrita corretiva. |
| Versão futura | A abertura falha sem downgrade, drop ou reset automático. |
| Corrupção/foreign key | A falha é explícita, redigida e o arquivo original é preservado. |
| Limpeza | Remove somente registros elegíveis e preserva trust/bloqueios/preferências. |
| Runtime Tauri | `Storage` inicia no `setup`, encerra no `Exit` e `greet` permanece o smoke test; os commands tipados da bridge entram na TASK 09. |

### Execução realizada

- `cargo test --manifest-path src-tauri/Cargo.toml`: 22 testes aprovados — 4 unitários do storage, 4 de domínio, 7 de lifecycle e 7 de storage/integração, além de doctests vazios.
- `npm test`: 4 arquivos e 14 testes TypeScript/Vue aprovados.
- `npm run typecheck`: aprovado.
- `npm run build`: aprovado; os testes não entram no bundle frontend.
- `cargo check --manifest-path src-tauri/Cargo.toml`: aprovado.
- `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`: aprovado.
- `git diff --check`: aprovado após a revisão final.

O smoke test Tauri completo não foi necessário para validar a camada offline: `greet` continua sem alteração de contrato e o `setup` foi validado por `cargo check`/runtime tests; a abertura usa o caminho Tauri apenas no hook da aplicação, enquanto os testes usam diretórios temporários fora do perfil do usuário.
