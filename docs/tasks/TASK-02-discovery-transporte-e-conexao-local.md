# TASK 02 — Decidir discovery, transporte e ciclo de conexão local

Status: decisão e POC base concluídos; interop avançada fica para as tasks de implementação

## Objetivo

Escolher a estratégia técnica para encontrar candidatos na rede local e estabelecer conexões diretas entre dispositivos Pulse, preservando a separação entre discovery, presença, pairing, trust e capabilities definida na TASK 01.

Esta task fecha uma decisão arquitetural e um plano de validação. Ela não adiciona sockets, dependências de rede, comandos Tauri, eventos IPC, criptografia de produção ou integração com os mocks atuais.

## Estado atual

- O produto é local-first e não usa cloud como requisito para comunicação entre dispositivos (`PRODUCT.md:27-33`).
- No momento desta decisão, discovery, pairing, transporte, persistência e eventos de produto ainda não existiam. A TASK 11 implementou posteriormente o browse mDNS e o registro transitório, mantendo pairing, transporte e eventos de produto fora do escopo (`PRODUCT.md:56-63`, `SYSTEM-DESIGN.md:5-7,78-93`).
- A arquitetura já separa UI Vue, bridge Tauri, domínio Rust, discovery/transport/protocol e serviços de recurso (`SYSTEM-DESIGN.md:138-146`). A UI não deve conhecer sockets, criptografia ou pacotes.
- `DiscoveryCandidate` contém um endpoint transitório, capabilities anunciadas, timestamps e expiração (`src/types/index.ts:86-109`); `Device` e `Presence` representam identidade conhecida e disponibilidade observada separadamente (`src/types/index.ts:116-131`).
- As transições canônicas já distinguem candidato `discovered/expired` e presença `unknown/online/stale/offline` (`src/types/index.ts:437-447`, `src-tauri/src/domain/mod.rs:46-81`). Não há estado de conexão ou serviço de networking implementado.
- `src-tauri/src/domain/` contém somente modelos puros; `src-tauri/src/lib.rs` registra apenas `greet`; `src-tauri/Cargo.toml` não possui dependência de discovery ou transporte.
- Os stores e componentes ainda usam `MockDevice`, `MockTransfer`, booleano `online` e copy demonstrativo (`src/stores/devices.ts:6-42`, `src/views/HomeView.vue:20-24`). Eles não serão adaptados nesta task.
- A validação exigida pelo índice é uma revisão do fluxo online/offline e um teste de viabilidade com duas instâncias Linux, sem código de produção (`TODO.md:37-46`).

## Brainstorm

### Critérios

1. Funcionar diretamente na LAN, sem servidor intermediário, relay ou NAT traversal como requisito.
2. Descobrir candidatos sem transformar nome, IP, porta, plataforma ou TXT em identidade confiável.
3. Transportar comandos, eventos, conteúdo leve e arquivos com controle de fluxo, concorrência e retomada possíveis nas tasks posteriores.
4. Permitir portas atribuídas pelo sistema e mudanças de endereço sem amarrar a identidade ao endpoint.
5. Ter um limite de implementação claro para Linux agora e adaptadores possíveis para macOS, Windows, Android e iOS depois.
6. Falhar de modo observável quando multicast, UDP, firewall, interface ou peer não estiverem disponíveis.

### Discovery

**mDNS + DNS-SD.** DNS-SD fornece enumeração de instâncias por PTR, resolução de host/porta por SRV e metadados pequenos por TXT ([RFC 6763](https://www.rfc-editor.org/rfc/rfc6763.html)). mDNS limita o domínio `.local` ao link local e usa UDP 5353 em `224.0.0.251`/`FF02::FB` ([RFC 6762](https://www.rfc-editor.org/rfc/rfc6762.html)). Isso atende à direção local-first e evita um registro central.

O serviço Pulse deve ser `_pulse._udp.local.` porque o transporte escolhido é QUIC sobre UDP. O anúncio deve conter apenas dados necessários para seleção e compatibilidade:

- nome de instância apresentado ao usuário, sujeito à resolução de conflitos do DNS-SD;
- versão maior do protocolo e versão do modelo;
- transporte (`quic`);
- plataforma declarada;
- capabilities anunciadas, limitadas ao vocabulário canônico;
- porta QUIC no registro SRV.

O anúncio não deve conter segredo, token, credencial, caminho local, conteúdo de Clipboard ou uma afirmação de trust. `DeviceId` persistente e fingerprint podem aparecer somente segundo a decisão de identidade/pairing da TASK 03; não são inferidos do nome mDNS.

O candidato precisa preservar todos os endpoints resolvidos, não apenas uma string: família IP, endereço, porta e índice da interface quando necessário para IPv6 link-local. O `DiscoveryEndpoint.value` atual permanece abstrato; a forma estruturada será definida na implementação de discovery.

### Transporte

**QUIC v1 via `quinn`.** QUIC fornece streams com controle de fluxo, estabelecimento de baixa latência, migração de caminho e conexão segura; os pacotes são transportados por UDP ([RFC 9000](https://www.rfc-editor.org/rfc/rfc9000.html)). A integração com TLS 1.3 é parte do protocolo ([RFC 9001](https://www.rfc-editor.org/rfc/rfc9001.html)), e `quinn` oferece uma implementação Rust portátil com `Endpoint` cliente/servidor ([documentação do quinn](https://docs.rs/quinn/latest/quinn/)).

O Pulse usará streams confiáveis para operações de produto. Datagrams QUIC não serão necessários para a primeira versão: dados de arquivos, comandos, eventos e conteúdo leve precisam de entrega e validação explícitas. O mapeamento entre streams, envelope, capabilities e mensagens pertence às TASKS 20 e 21.

O endpoint QUIC será alocado em porta dinâmica pelo sistema. O serviço DNS-SD publica a porta atual; após uma mudança de interface, endereço ou reinício, a instância republica e os consumidores resolvem novamente. Não haverá uma porta fixa obrigatória nem uma regra que dependa de IP ou MAC.

### Ciclo de conexão

O ciclo interno de conexão não será uma nova entidade canônica nesta task. Ele ficará atrás de um `ConnectionManager` no Rust e poderá usar estados internos `idle`, `resolving`, `connecting`, `handshaking`, `connected`, `closing` e `failed`. A bridge futura traduzirá esses resultados em eventos e estados de presença sem fazer a UI conhecer QUIC.

O fluxo de alto nível é:

```mermaid
flowchart LR
  A["anúncio mDNS/DNS-SD"] --> B["candidato transitório"]
  B --> C["presença observada"]
  C --> D["pairing explícito"]
  D --> E["trust válido"]
  E --> F["handshake QUIC/TLS"]
  F --> G["sessão autorizada por capability"]
```

Um candidato não confiável pode ser observado e, quando a TASK 03 definir o fluxo, contatado somente para uma etapa limitada de identificação/pairing. Ele nunca pode iniciar transferência, ler/escrever Clipboard, controlar mídia ou executar comando remoto. Um anúncio válido não prova identidade; uma conexão QUIC estabelecida não concede capability.

### Temporização e reconexão

O relógio monotônico será usado para expiração, heartbeat, timeout e backoff. Timestamps de domínio continuam em UTC/RFC 3339 conforme a TASK 01.

| Item | Política inicial | Resultado observável |
| --- | --- | --- |
| Registro DNS-SD | Honrar TTL recebido; usar 120 s como perfil inicial para registros de host/SRV, conforme a orientação do RFC 6762 | `DiscoveryCandidate` expira pelo TTL; encerramento limpo envia goodbye quando suportado |
| Resolução/conexão | 5 s por tentativa de endpoint | falha de conexão; tentar o próximo endpoint quando houver |
| Handshake QUIC/aplicação | 10 s, incluindo negociação de versão e identidade | `failed`/`transport-blocked`, sem alterar trust |
| Keep-alive | ping de aplicação a cada 15 s em sessão que precisa permanecer observável | atualização de `lastSeenAt` apenas após resposta válida |
| Presença stale | 30 s sem liveness válida | `Presence = stale` |
| Presença offline | 60 s sem liveness, fechamento explícito ou erro definitivo | `Presence = offline`; trust permanece como estava |
| Idle QUIC | limite inicial de 60 s, maior que o intervalo de liveness | sessão encerrada pelo transport se não houver tráfego/keep-alive |
| Reconexão confiável | 1, 2, 4, 8, 16, 32 e 60 s, com jitter de ±25%; reinicia após sucesso | não reconectar automaticamente candidatos não confiáveis |
| Candidato sem anúncio | expira pelo TTL efetivo; no máximo 5 min como limite de proteção contra cache defeituoso | `DiscoveryCandidate = expired`, sem revogar trust |

Os valores são política inicial para o POC e podem ser ajustados com medições de bateria, tráfego e falsos offline. O contrato não deve transformar esses números em copy fixa da UI.

### Escopo de rede e ausência de rede

- O escopo padrão é o link local de cada interface ativa, sem relay, cloud, STUN, TURN, roteamento entre VLANs ou travessia de NAT.
- Loopback fica excluído no modo de produto e só pode ser habilitado por uma fixture de teste. Interfaces VPN, containers e redes virtuais ficam desabilitadas por padrão até haver uma política explícita.
- IPv4 e IPv6 devem ser observados quando disponíveis. Endereço IPv6 link-local deve carregar o índice da interface; não se deve tentar conectar usando apenas o texto do endereço.
- Em uma máquina multihomed, cada resposta deve ser associada à interface que a recebeu. O RFC 6762 exige que endereços anunciados sejam válidos na interface correspondente e recomenda tratar interfaces de modo explícito.
- Sem interface elegível, o discovery entra em estado interno `no-interface`; não há erro fatal nem candidato novo. Candidatos existentes expiram e dispositivos conhecidos passam a `stale`/`offline` conforme os relógios.
- Se mDNS funcionar mas QUIC for bloqueado por firewall, isolamento Wi-Fi ou política de UDP, o candidato continua distinto de `connected`: registrar `transport-blocked`/`peer-unreachable` internamente e manter `trust` intacto.
- Quando uma interface retornar, reinicializar o browse/registro, resolver endpoints atuais e aplicar o backoff somente a dispositivos já confiáveis. Reaparecer na rede nunca reativa trust revogado.

## Decisões

- **Discovery escolhido:** mDNS + DNS-SD, com serviço `_pulse._udp.local.` e registros PTR/SRV/TXT mínimos.
- **Transporte escolhido:** QUIC v1 via `quinn`, com streams confiáveis e porta dinâmica publicada por DNS-SD.
- **Boundary de implementação:** discovery e transporte serão traits/adapters Rust atrás de `discovery/`, `protocol/` e `device/`; a UI só consumirá modelos/eventos pela bridge futura.
- **Candidato versus confiável:** anúncio, nome, endpoint, plataforma e capabilities anunciadas são não confiáveis. A operação exige trust válido e capability concedida; pairing e identidade criptográfica são TASK 03.
- **Identidade:** não usar IP, porta, nome mDNS, instância ou CandidateId como `DeviceId`. Mudança de endpoint atualiza presença/endpoints, não cria identidade nova quando a identidade autenticada permanece a mesma.
- **Portas:** usar porta UDP dinâmica para QUIC; não reservar uma porta fixa obrigatória. O registro SRV é a fonte de endpoint transitório e deve ser resolvido novamente após mudança de rede.
- **Escopo:** LAN/link-local apenas na primeira versão. Conexões fora da LAN, relay e NAT traversal são fora de escopo.
- **Ciclo:** sessões QUIC e estados de conexão são internos ao Rust até a TASK 05 definir o contrato da bridge; `Presence` continua separado do resultado de pairing/trust.
- **Falhas:** offline, ausência de interface, candidato expirado, conexão bloqueada e trust revogado são resultados diferentes. Nenhum erro de rede pode sozinho revogar trust.
- **Segurança adiada, limite não:** QUIC/TLS fornece o canal criptográfico, mas a verificação de identidade, armazenamento de chaves, pairing, revogação, anti-replay e capabilities pertencem à TASK 03 e às TASKS 20-22. Nunca aceitar qualquer certificado em produção.

### Alternativas rejeitadas

| Alternativa | Decisão | Motivo |
| --- | --- | --- |
| Broadcast UDP próprio | Rejeitada | Não oferece enumeração/resolução padronizada, aumenta colisões e exige resolver interface, TTL, IPv4/IPv6 e conflito manualmente. UDP permanece apenas como base padronizada do mDNS e do QUIC. |
| SSDP/UPnP | Rejeitada | Mais orientado a ecossistemas de dispositivos e HTTP; não traz vantagem para o domínio local do Pulse e aumenta superfície de parsing/semântica. |
| TCP + TLS 1.3 | Mantida como alternativa futura | É simples e amplamente disponível, mas exige multiplexação/framing próprios, sofre head-of-line no stream e não oferece migração de conexão equivalente. Só deve ser implementada se testes reais mostrarem bloqueio recorrente de UDP. |
| WebSocket | Rejeitada como transporte nativo | Adiciona handshake HTTP e não resolve discovery, identidade ou pairing; pode ser uma camada de compatibilidade futura para navegador, não o canal Rust entre peers. |
| WebRTC | Rejeitada | É adequada quando há navegador, mídia, ICE/STUN/TURN ou travessia de NAT. A especificação exige sinalização externa e acrescenta SDP/ICE/DTLS; isso é desnecessário para peers na mesma LAN ([W3C WebRTC](https://www.w3.org/TR/webrtc/), [RFC 8445](https://www.rfc-editor.org/rfc/rfc8445.html)). |
| Porta TCP/UDP fixa | Rejeitada como requisito | Facilita regras de firewall, mas cria conflitos entre instâncias e acopla o contrato a uma porta estática. DNS-SD permite publicar a porta atribuída pelo sistema. |

## Plano de implementação

Esta task não implementa os itens abaixo; ela deixa a sequência pronta para as tasks responsáveis.

1. Criar um adapter de discovery com browse/register/resolve, filtro de interfaces, preservação de endpoints IPv4/IPv6 e expiração baseada no TTL.
2. Validar `mdns-sd` como primeira implementação Rust, mantendo a trait aberta para backend nativo por plataforma. A documentação do crate declara suporte a responder/querier, IPv4/IPv6 e Linux/macOS/Windows ([mdns-sd](https://docs.rs/mdns-sd/latest/mdns_sd/)). Android e iOS exigirão adapter próprio quando entrarem no escopo.
3. Criar um `ConnectionManager` Rust que recebe candidatos/endpoints, aplica timeout por endpoint, abre QUIC e mantém o ciclo interno de estados.
4. Definir o `Transport`/`Protocol` sem expor tipos de `quinn` ao domínio ou à UI. Framing, envelope, capabilities e negociação de mensagens ficam nas TASKS 20 e 21.
5. Adicionar liveness e reconexão com relógio monotônico, backoff e jitter; emitir fatos de domínio/diagnóstico sem converter `offline` em `revoked`.
6. Só depois da TASK 03 liberar pairing e recursos; só depois das TASKS 05/09 adaptar comandos/eventos para Vue.
7. Manter o modo mockado atual até a TASK 10; não alterar stores, rotas, placeholders ou o command `greet` como efeito colateral.

## Execução paralela

A investigação foi paralelizada em dois recortes sem sobreposição:

- **Inventário local:** leitura de `PRODUCT.md`, `DESIGN.md`, `SYSTEM-DESIGN.md`, `TODO.md`, TASK 01, tipos TypeScript, modelos Rust, stores e configuração Tauri. O resultado confirmou que não existe networking ativo e apontou a lacuna de modelo de conexão.
- **Pesquisa técnica:** comparação de mDNS/DNS-SD, QUIC/`quinn`, TCP/TLS e WebRTC usando RFCs, documentação do crate e especificações oficiais. O resultado sustentou mDNS + QUIC para a primeira decisão.

A consolidação, a decisão e a integração documental permanecem sequenciais neste arquivo. Não há implementação paralela: nenhum subagente edita código ou este plano.

## Integração

- A TASK 11 deve implementar discovery sem assumir trust e sem adaptar diretamente os mocks.
- A TASK 12 deve derivar presença de observações/heartbeat e distinguir `stale`, `offline`, `transport-blocked` e ausência de rede.
- A TASK 03 deve definir identidade e verificação de peer antes de qualquer operação autorizada; a sessão QUIC não é autorização.
- A TASK 05 deve definir os commands/eventos da bridge sem expor `quinn`, `SocketAddr`, TXT ou detalhes de interface à UI.
- As TASKS 20 e 21 devem definir canal seguro, envelope, versionamento, capabilities e validação de payloads.
- A TASK 10 só pode conectar candidatos/presença reais ao Vue depois de existir uma fonte de estado explícita; os fixtures atuais continuam demonstrativos.

## Critérios de conclusão

- [x] Discovery decidido como mDNS/DNS-SD em escopo link-local, com serviço, registros, interfaces, IPv4/IPv6 e limite de confiança documentados.
- [x] Transporte decidido como QUIC v1/`quinn`, com streams, porta dinâmica e dependência de UDP explicitados.
- [x] Alternativas rejeitadas e o motivo de não escolher TCP/TLS ou WebRTC como primeira implementação registrados.
- [x] Candidato descoberto, presença observada, pairing, trust e capability separados em fluxo explícito.
- [x] Timeouts, TTL, heartbeat, expiração, ausência de interface, firewall e reconexão descritos com resultados distintos.
- [x] POC executado em duas instâncias Linux com anúncio/consulta mDNS, portas dinâmicas, QUIC, reinício e pelo menos IPv4; IPv6 e interfaces múltiplas permanecem cenários adicionais.
- [x] Nenhum código de produção, dependência, capability Tauri, comando ou alteração nos mocks foi introduzido nesta task.

## Validação

### Evidência realizada

- Leitura cruzada de `PRODUCT.md`, `DESIGN.md`, `SYSTEM-DESIGN.md`, `TODO.md`, TASK 01, modelos TypeScript/Rust, stores e configuração Tauri.
- Pesquisa em fontes primárias: [RFC 6762](https://www.rfc-editor.org/rfc/rfc6762.html), [RFC 6763](https://www.rfc-editor.org/rfc/rfc6763.html), [RFC 9000](https://www.rfc-editor.org/rfc/rfc9000.html), [RFC 9001](https://www.rfc-editor.org/rfc/rfc9001.html), [RFC 8445](https://www.rfc-editor.org/rfc/rfc8445.html), [W3C WebRTC](https://www.w3.org/TR/webrtc/), [quinn](https://docs.rs/quinn/latest/quinn/) e [mdns-sd](https://docs.rs/mdns-sd/latest/mdns_sd/).
- O ambiente possui `avahi-browse`/`avahi-publish`, mas o daemon Avahi não está em execução. Isso foi contornado no POC usando a implementação Rust `mdns-sd`, sem adicionar dependência ao Pulse.
- O POC descartável em `/tmp/pulse-task02-poc.ujR4xT` compilou `mdns-sd 0.21`, `quinn 0.11`, Tokio, Rustls e `rcgen` fora do repositório. O cliente encontrou `_pulse._udp.local.`, resolveu o TXT `proto=1`, conectou ao endpoint QUIC e recebeu `pulse-task02-echo`.
- Em uma execução na interface LAN `enp42s0` (`192.168.2.2`), o serviço foi resolvido em uma porta dinâmica `53045` e o eco terminou com status `0`. Dois ciclos adicionais de reinício resolveram novas portas `34244` e `53356`, ambos com eco bem-sucedido.
- O POC usou certificado autoassinado e verificador que aceita qualquer certificado somente para viabilidade de transporte; isso não é uma decisão de segurança de produção. IPv6, duas máquinas físicas/VMs, firewall e encerramento abrupto permanecem validações das tasks 03, 11 e 12.

### POC base realizado e variações futuras

Executar em duas instâncias Linux, preferencialmente duas máquinas/VMs na mesma LAN e depois duas instâncias no mesmo host:

1. Cada processo obtém uma porta QUIC dinâmica e registra uma instância `_pulse._udp.local.` com TXT mínimo.
2. Cada processo navega, resolve PTR/SRV/TXT e A/AAAA, preserva os endpoints e cria exatamente um candidato por anúncio transitório.
3. O cliente abre QUIC na porta resolvida, completa o handshake de teste e troca uma mensagem de eco; nenhum recurso real ou capability de produção é habilitado.
4. Encerrar um processo normalmente e verificar goodbye/expiração; encerrá-lo abruptamente e verificar expiração por TTL.
5. Reiniciar o processo, trocar a porta e confirmar que o mesmo anúncio transitório atualiza endpoint sem criar trust.
6. Bloquear UDP/QUIC no firewall, desligar a interface e reativá-la; confirmar `transport-blocked`/`offline` e reconexão com backoff.
7. Repetir com IPv6 quando disponível, preservando o índice de interface para endereços link-local.

O POC deve ser descartável, ficar fora do repositório e usar certificados/identidades de teste explicitamente marcados como inseguros. O POC base desta task foi concluído; as variações de interop e falha de rede permanecem critérios das tasks de implementação.

## Dependências e limites

Esta decisão desbloqueia as TASKS 11 e 12, mas não autoriza implementação de produção isoladamente. Permanecem fora desta task: threat model completo e armazenamento de chaves (TASK 03), persistência (TASK 04/08), contrato da bridge (TASK 05/09), runtime de serviços (TASK 07), pairing real (TASK 16), trust e revogação (TASK 17), política de capabilities (TASK 18), canal seguro de peers confiáveis (TASK 20), envelope/capabilities de mensagens (TASK 21), validação de abuso (TASK 22) e recursos de arquivos/Clipboard/mídia/comandos.
