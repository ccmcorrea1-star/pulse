# Pulse — Produto

## O que é

Pulse é um aplicativo desktop Linux para integrar dispositivos próximos diretamente pela rede local. A direção do produto é semelhante à de ferramentas como LocalSend e KDE Connect, mas com uma central de controle compacta para descobrir e parear dispositivos confiáveis, compartilhar conteúdo e acompanhar o estado dessas interações sem depender de cloud.

O repositório está na versão `0.1.0` e contém a fundação navegável do aplicativo. A rede local, os dispositivos e as transferências exibidos hoje são dados demonstrativos; não há descoberta, pareamento ou transporte de dados ativo.

## Objetivos

- Tornar claro quais dispositivos estão disponíveis e qual dispositivo está em contexto.
- Permitir compartilhamento local e direto de arquivos, pastas, texto e links.
- Dar controle explícito sobre confiança, autorização e estado das transferências.
- Reunir Clipboard, mídia, notificações, controle remoto e comandos em uma experiência coerente por dispositivo.
- Manter o produto local-first: sem cloud como requisito para a comunicação entre dispositivos.

## Público e situações de uso

O público principal é quem administra os próprios dispositivos — ou uma pequena rede doméstica/de equipe — e quer mover conteúdo entre computador, telefone e outros endpoints próximos. Os usos previstos incluem:

- enviar um arquivo ou pasta para um dispositivo pareado;
- compartilhar texto ou link rapidamente;
- acompanhar, pausar e retomar transferências;
- sincronizar ou enviar itens do Clipboard com controle por dispositivo;
- consultar mídia, controlar um dispositivo e executar comandos locais quando essas capacidades forem autorizadas.

## Princípios de produto

- **Local e direto:** a comunicação entre dispositivos deve acontecer na rede local, sem um serviço remoto intermediário como requisito.
- **Confiança explícita:** um dispositivo não deve ganhar acesso apenas por aparecer na rede; pareamento, autorização e revogação precisam ser compreensíveis.
- **Estado visível:** origem, destino, progresso, erro e resultado devem ser legíveis sem depender somente de cor.
- **Capacidade por dispositivo:** cada recurso pode ser oferecido, solicitado, concedido ou revogado separadamente.
- **Progresso honesto:** a interface deve identificar dados mockados e estados ainda não conectados ao backend.

## Escopo atual do repositório

### Implementado

- Shell desktop Tauri 2 com uma janela principal configurada para `1280 × 800`, mínimo de `960 × 640`.
- Frontend Vue 3 + TypeScript + Vite, montado por `src/main.ts`.
- Navegação global para `Início`, `Transferências`, `Histórico` e `Configurações`.
- Lista mockada com três dispositivos, estados online/offline e links para o contexto de cada dispositivo.
- Rotas aninhadas de dispositivo para `Visão geral`, `Arquivos`, `Clipboard`, `Mídia` e `Controle`.
- Stores Pinia efêmeras para aplicação, dispositivos e transferências.
- Dois registros mockados de transferência, incluindo progresso e fila visual na página inicial.
- Tela de configuração capaz de testar a comunicação Vue ↔ Rust por meio do command `greet`.
- Infraestrutura bridge tipada e bootstrap Vue para consultar info/snapshot público, observar eventos/status do runtime e manter o estado de infraestrutura nos stores; o estado de produto continua não configurado.
- Interface dark, compacta e responsiva em desktop, com adaptação básica para larguras menores.
- Fundação de persistência SQLite local no Rust, com schema versionado, migrations forward-only e proteção contra corrupção/incompatibilidade; os stores Vue ainda não são hidratados por ela.

### Estruturado/preparado

- O router já reserva os destinos das áreas funcionais por dispositivo.
- `src-tauri/src/` contém diretórios preparados para `discovery`, `pairing`, `device`, `protocol`, `transfer`, `clipboard` e `media`, ainda sem implementação de domínio.
- O contrato canônico de domínio já está estruturado em TypeScript e em modelos puros Rust; os stores usam adaptadores de apresentação e mantêm fixtures isoladas no boundary de desenvolvimento enquanto não há dados de produto.
- A configuração de capabilities Tauri existe e hoje concede somente `core:default`.

### Ainda não implementado

- Descoberta de dispositivos na rede local.
- Pareamento, identidade, confiança e revogação.
- Transferência real de arquivos ou pastas, incluindo seleção, fila, pausa, retomada e cancelamento.
- Clipboard entre dispositivos, envio de texto/links e histórico persistente.
- Notificações, mídia, controle remoto e comandos.
- Dados reais de dispositivos/transferências, persistência funcional de produto, identidade, histórico, sincronização de eventos de produto e qualquer protocolo de rede.

## Funcionalidades e roadmap

O roadmap abaixo descreve direção funcional. Ele não representa código já disponível.

| Capacidade | Estado atual | Próxima direção |
| --- | --- | --- |
| Dispositivos | Fixture de desenvolvimento, fonte vazia fora de DEV e rotas de contexto | Discovery local, detalhes e atualização de presença |
| Pareamento e confiança | Não existe; apenas estrutura de módulos | Pareamento explícito, identidade verificável e permissões por capacidade |
| Arquivos e pastas | Aba reservada; sem picker ou transporte | Envio local direto com progresso, fila, pausa e retomada |
| Texto e links | Não há fluxo funcional | Tratar como conteúdo leve dentro do envio local |
| Clipboard | Aba reservada; sem leitura ou escrita remota | Compartilhamento controlado de texto, links e tipos suportados |
| Transferências | Preview de fixture de desenvolvimento e rota vazia | Serviço de transferência, estados reais e cancelamento/retomada |
| Histórico | Rota e estado vazio; schema local preparado | Registro local de eventos, origem, destino e resultado via bridge |
| Notificações | Não existe | Avisos locais para pedidos, conclusão e falha |
| Mídia | Rota com placeholder | Leitura e controle de mídia sob capability explícita |
| Controle e comandos | Rotas reservadas | Ações remotas limitadas, auditáveis e autorizadas |

### Ordem sugerida

1. Discovery local e pareamento confiável.
2. Modelo de dispositivos, capabilities e eventos.
3. Transferência de arquivos/pastas e conteúdo leve.
4. Clipboard, notificações e histórico persistente.
5. Mídia, controle e comandos, depois de definidos os limites de segurança.

Os detalhes técnicos dessa sequência pertencem ao [SYSTEM-DESIGN.md](SYSTEM-DESIGN.md); as decisões de interface pertencem ao [DESIGN.md](DESIGN.md).

## Limites e não objetivos atuais

- O projeto não oferece integração em cloud nem precisa dela para a direção local-first.
- A implementação atual não deve ser tratada como prova de segurança, criptografia ou transporte de produção.
- Não há suporte funcional para Android, iOS ou Windows neste repositório; esses sistemas aparecem apenas como plataformas previstas no tipo de dispositivo e na direção de interoperabilidade.
- `10-pulse-resumo.html` é um protótipo HTML legado e não é o ponto de entrada do app atual.
