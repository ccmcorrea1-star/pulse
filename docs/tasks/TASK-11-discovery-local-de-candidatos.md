# TASK 11 — Implementar discovery local de candidatos

Status: concluída; adapter mDNS/DNS-SD e registro transitório implementados sem bridge de produto

## Objetivo

Encontrar anúncios Pulse na rede local e materializá-los como candidatos transitórios, sem conceder identidade, pairing, trust ou capabilities autorizadas. A task entrega a primeira fronteira executável do módulo `discovery/`, preservando o modo demonstrativo da UI e deixando presença/heartbeat para a TASK 12.

## Estado atual

- `TODO.md:106-113` define a TASK 11 como a primeira task da fase de discovery, com dependências no runtime e na bridge já concluídos.
- `docs/tasks/TASK-02-discovery-transporte-e-conexao-local.md` decide mDNS/DNS-SD no serviço `_pulse._udp.local.`, registros TXT mínimos, escopo link-local, endpoints IPv4/IPv6 e TTL inicial de 120 segundos.
- `src/types/index.ts:86-109` e `src-tauri/src/domain/mod.rs:196-222` já possuem `DiscoveryCandidate`, `DiscoveryEndpoint`, capabilities anunciadas e os estados `discovered/expired`; `CandidateId` não é `DeviceId`.
- `src-tauri/src/runtime/mod.rs:5-16,132-138` já reserva `ServiceKind::Discovery` e o lifecycle `RuntimeService`; a implementação desta task registra `DiscoveryService` no bootstrap de `src-tauri/src/lib.rs`.
- `src-tauri/src/bridge/mod.rs` expõe somente infraestrutura e `ProductState::NotConfigured`; esta task não cria command ou evento de produto e não adapta os stores Vue.
- O crate `mdns-sd` 0.21.0 fornece `ServiceDaemon::browse`, `ServiceEvent::ServiceResolved/ServiceRemoved`, `ResolvedService` com porta, endereços com escopo IPv6 e propriedades TXT. A API foi conferida na documentação oficial do crate e no registry antes da implementação.

## Brainstorm

| Alternativa | Avaliação | Decisão |
| --- | --- | --- |
| Fazer polling de `avahi-browse` ou de comandos do sistema | Acopla o produto a um daemon/CLI externo, dificulta testes e não preserva a fronteira Rust. | Rejeitada |
| Implementar um protocolo UDP próprio | Duplica mDNS/DNS-SD e aumenta a superfície de parsing e compatibilidade. | Rejeitada |
| Expor `mdns-sd` diretamente para Vue | Vaza serviço, TXT, IP e detalhes de interface para a UI. | Rejeitada |
| Adapter mDNS + parser/registro puro testável | Permite integração real e testes com peers falsos sem rede, mantendo candidatos separados de confiança. | Escolhida |
| Excluir candidato imediatamente ao `ServiceRemoved` | Perde o estado terminal canônico `expired` e dificulta observabilidade. | Rejeitada |
| Reabrir candidato expirado sem nova resolução válida | Pode transformar cache antigo em presença atual. | Rejeitada; só resolução válida cria/atualiza candidato |

## Decisões

1. O serviço descoberto será `_pulse._udp.local.` e somente anúncios com `proto=1`, `model=1`, `transport=quic` e `platform` canônico serão aceitos.
2. O nome apresentado será derivado do nome da instância DNS-SD; ele é texto de apresentação, não prova de identidade.
3. `CandidateId` será determinístico dentro de uma geração de discovery a partir do fullname mDNS e nunca será convertido em `DeviceId`; uma reaparição após expiração recebe nova geração.
4. O registro aceitará apenas endereços não-loopback, preservará múltiplos endpoints e manterá o escopo IPv6 no valor serializado do endpoint.
5. Capabilities TXT desconhecidas, plataforma inválida, versão incompatível, ausência de endereço, porta zero ou serviço fora do tipo Pulse serão descartados como anúncios inválidos.
6. Uma resolução válida cria ou atualiza o candidato como `discovered`; remoção e expiração alteram-no para `expired`. Expiração usa o TTL inicial de 120 segundos documentado na TASK 02, com relógio injetável nos testes.
7. O serviço real roda em uma thread de consumo do receiver do `mdns-sd`; `stop` encerra browse, daemon e thread. Falha de inicialização é mapeada para `InitializationFailed` pelo runtime, sem expor erro da crate à bridge.
8. A task não implementa heartbeat, presença, pairing, trust, capabilities concedidas, QUIC, persistência de candidatos, bridge de produto ou UI.

## Plano de implementação

1. Adicionar `mdns-sd = "0.21.0"` ao Cargo e criar `src-tauri/src/discovery/mod.rs`.
2. Implementar tipos internos de anúncio/endereço, validação de TXT, conversão de `ResolvedService` e formatação de endpoints IPv4/IPv6.
3. Implementar `CandidateRegistry` puro com deduplicação por fullname, atualização de endpoint/metadados, `ServiceRemoved` e expiração controlada por `Instant`.
4. Implementar `DiscoveryService` como `RuntimeService`, iniciando browse no `start` e fazendo cleanup no `stop`, sem publicar dados na bridge nesta task.
5. Registrar o módulo e o serviço no bootstrap Tauri, mantendo o runtime parcial e o snapshot público sem estado de produto.
6. Adicionar testes Rust para anúncio válido, duplicado, inválido, fora do tipo, sem endereço, remoção, TTL, múltiplos endpoints, IPv6 com escopo e separação de confiança.
7. Atualizar `TODO.md`, `PRODUCT.md` e `SYSTEM-DESIGN.md` apenas após implementação e validação, marcando discovery como implementado/estruturado somente no limite real entregue.

## Execução paralela

Não há paralelismo de escrita necessário: o módulo de discovery, o registro do runtime e os testes compartilham contratos próximos. A pesquisa técnica foi feita separadamente da leitura do código; a implementação será integrada sequencialmente para evitar divergência entre o adapter, o lifecycle e os critérios.

## Integração

- O runtime inicia o browse mDNS, mas não o trata como serviço pronto de produto nem altera `BridgeSnapshot`.
- A TASK 12 consumirá o registro/fluxo de observações para derivar `online`, `stale` e `offline`; expiração de candidato não equivale a presença offline de um dispositivo conhecido.
- A TASK 13 poderá persistir dispositivos conhecidos sem persistir automaticamente anúncios transitórios.
- As TASKS 15–19 definirão identidade, pairing, trust e capabilities; nenhum TXT ou endpoint desta task concede autorização.
- A TASK 14 será responsável por adaptar a UI quando houver DTO/evento de discovery apropriado.

## Critérios de conclusão

- [x] O serviço `_pulse._udp.local.` é navegado em runtime Rust sem comando de produto.
- [x] Anúncios válidos criam/atualizam candidatos e duplicados não criam registros adicionais.
- [x] Anúncios inválidos, fora do escopo, sem endpoint ou incompatíveis são descartados sem panic.
- [x] `ServiceRemoved` e TTL produzem `DiscoveryCandidateState::Expired`.
- [x] Endpoints IPv4/IPv6 são preservados, incluindo escopo de IPv6 link-local.
- [x] Candidato, identidade, presença, pairing, trust e capability permanecem separados.
- [x] O cleanup do runtime encerra browse/daemon/thread sem vazamento observável.
- [x] Testes Rust e validações do repositório passam.

## Validação

- `cargo test --manifest-path src-tauri/Cargo.toml`
- `cargo check --manifest-path src-tauri/Cargo.toml`
- `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`
- `npm run typecheck`
- `npm test`
- `npm run build`
- `npm run test:rust`
- `git diff --check`
- Smoke test opcional com duas instâncias Linux anunciando `_pulse._udp.local.`, quando multicast estiver disponível; ausência de interface/daemon não deve impedir os testes unitários do parser/registro.

### Execução realizada

- `cargo fetch --manifest-path src-tauri/Cargo.toml` — passou com `mdns-sd 0.21.0` e dependências resolvidas.
- `cargo test --manifest-path src-tauri/Cargo.toml` — passou: 31 testes Rust, 0 falhas.
- `cargo check --manifest-path src-tauri/Cargo.toml` — passou.
- `cargo fmt --manifest-path src-tauri/Cargo.toml` — passou.
- Testes de discovery cobrem anúncio válido, deduplicação, atualização de endpoint, anúncio inválido, capability desconhecida, ausência de endereço, remoção, TTL, nova geração após expiração, múltiplos endpoints e IPv6 com escopo.
- O smoke test multicast entre duas instâncias não foi repetido nesta task; a POC da TASK 02 já validou a viabilidade do crate na LAN, enquanto a integração desta task permanece sem QUIC ou bridge de produto.

### Evidências e fontes

- [`TODO.md:106-113`](/home/kyle/Documentos/projeto1/TODO.md:106)
- [`docs/tasks/TASK-02-discovery-transporte-e-conexao-local.md`](/home/kyle/Documentos/projeto1/docs/tasks/TASK-02-discovery-transporte-e-conexao-local.md)
- [`src-tauri/src/domain/mod.rs:196-222`](/home/kyle/Documentos/projeto1/src-tauri/src/domain/mod.rs:196)
- [`src-tauri/src/runtime/mod.rs:5-16`](/home/kyle/Documentos/projeto1/src-tauri/src/runtime/mod.rs:5)
- [Documentação oficial do `mdns-sd`: browse, daemon e eventos](https://docs.rs/mdns-sd/0.21.0/mdns_sd/)
- [Documentação oficial de `ResolvedService` e endpoints com escopo](https://docs.rs/mdns-sd/0.21.0/mdns_sd/struct.ResolvedService.html)
