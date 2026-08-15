# Pulse — TODO

Este é o índice principal do desenvolvimento futuro do Pulse. A fundação Tauri 2 + Vue 3 + TypeScript + Vite, o shell navegável, as rotas, os stores Pinia efêmeros, os mocks visuais, o command Rust `greet`, a bridge tipada de infraestrutura, o lifecycle estrutural do runtime e a fundação de storage SQLite local são considerados concluídos. Mocks e placeholders continuam sendo apenas suporte de desenvolvimento até que uma task os substitua por estado ou integração real.

Quando uma task for iniciada, criar seu plano em `docs/tasks/TASK-XX-nome.md`. O arquivo detalhado deve ser criado somente nesse momento; `docs/tasks/` é mantido como diretório de entrada para esses planos.

## Processo obrigatório para iniciar uma task

Antes de implementar qualquer task, é obrigatório:

1. Ler a task, a documentação relacionada e o código existente.
2. Fazer investigação e planejamento, usando subagentes com responsabilidades distintas quando houver paralelismo real.
3. Registrar evidências concretas com caminho e linhas relevantes; consultar documentação oficial quando a decisão depender do comportamento de uma biblioteca.
4. Consolidar divergências e decisões no plano da task antes de alterar código.

Cada plano em `docs/tasks/TASK-XX-nome.md` deve conter, no mínimo:

- `## Objetivo`;
- `## Estado atual`;
- `## Brainstorm`;
- `## Decisões`;
- `## Plano de implementação`;
- `## Execução paralela`;
- `## Integração`;
- `## Critérios de conclusão`;
- `## Validação`.

O agente principal é responsável pela consolidação, integração, revisão final, testes e atualização da documentação. A implementação só começa depois que o plano estiver consolidado. Não criar subagentes artificiais nem editar o mesmo arquivo em paralelo sem ownership explícito.

As dependências abaixo são precedentes de produto ou arquitetura. A validação de cada task deve preservar o comportamento já existente enquanto a capacidade correspondente ainda não estiver conectada.

## Fase 1 — Contratos e decisões fundamentais

- [x] TASK 01 — Fechar os modelos de domínio e seus estados
  - Objetivo: Definir a linguagem comum que será usada pelo Rust, pela bridge e pelo Vue.
  - Escopo: Dispositivo, presença, pairing, trust, capability, transferência, conteúdo leve, histórico, notificação, mídia e comando remoto; IDs, estados, transições e timestamps.
  - Dependências: Fundação Tauri/Vue concluída.
  - Critérios de conclusão: Os modelos, estados válidos e invariantes estão documentados de forma consistente entre frontend e backend, sem depender dos tipos mockados atuais.
  - Validação: Revisão cruzada com `PRODUCT.md`, `DESIGN.md` e `SYSTEM-DESIGN.md`; conferir que cada estado visível tem uma representação textual honesta.

- [x] TASK 02 — Decidir discovery, transporte e ciclo de conexão local
  - Objetivo: Escolher a estratégia técnica para encontrar peers e manter conexões diretas na rede local.
  - Escopo: Mecanismo de discovery, transporte, portas e escopos de rede, timeouts, reconexão, ausência de rede e compatibilidade futura entre plataformas.
  - Dependências: TASK 01.
  - Critérios de conclusão: Há uma decisão registrada, alternativas rejeitadas e limites claros para discovery de candidato versus dispositivo confiável.
  - Validação: Revisão arquitetural de fluxo online/offline e teste de viabilidade em duas instâncias Linux sem introduzir código de produção.

- [x] TASK 03 — Definir threat model, identidade, trust e capabilities
  - Objetivo: Estabelecer as regras de segurança que antecedem qualquer comunicação real.
  - Escopo: Ameaças locais, identidade de peer, autenticação de pairing, criptografia, armazenamento de segredos, revogação, estados de capability e limites para comandos remotos.
  - Dependências: TASK 01 e TASK 02.
  - Critérios de conclusão: Existe uma política de segurança verificável, com matriz de capabilities, decisões de aprovação/negação/revogação e comportamento para perda de confiança.
  - Validação: Revisão de segurança cobrindo spoofing, replay, peer não pareado, payload malformado, vazamento de caminho e execução não autorizada.

- [x] TASK 04 — Decidir persistência, migrações e retenção local
  - Objetivo: Escolher como o Pulse armazenará estado confiável e histórico sem depender de cloud.
  - Escopo: Backend de armazenamento, esquema e migrações, dados sensíveis, retenção de Clipboard/histórico, corrupção, reset e recuperação.
  - Dependências: TASK 01 e TASK 03.
  - Critérios de conclusão: A estratégia escolhida define o que persiste, por quanto tempo, como evolui e o que nunca deve ser armazenado.
  - Validação: Revisão de cenários de primeiro uso, upgrade, downgrade não suportado, falha de escrita e remoção de dados.

- [x] TASK 05 — Definir o contrato da bridge Rust ↔ Vue
  - Objetivo: Padronizar como a UI solicita intenções e recebe estado e eventos de domínio.
  - Escopo: Comandos, eventos, payloads, erros, versionamento, validação de entrada, ciclo de vida dos listeners e comportamento da prévia web.
  - Dependências: TASK 01, TASK 03 e TASK 04.
  - Critérios de conclusão: O contrato separa UI de transporte, não expõe detalhes internos de sockets e define respostas para sucesso, erro, loading, stale e offline.
  - Validação: Revisar os contratos com cenários de erro e garantir que o command `greet` continue apenas como smoke test da fundação.

- [x] TASK 06 — Preparar a base de testes e fixtures
  - Objetivo: Criar a capacidade de validar domínios, bridge, UI e integrações sem depender de dispositivos reais em toda execução.
  - Escopo: Ferramentas de teste Rust/TypeScript/Vue, fixtures de domínio, relógio controlável, peers falsos, dados de erro e comandos de validação.
  - Dependências: TASK 01 e TASK 05.
  - Critérios de conclusão: A suíte mínima executa localmente, tem fixtures versionadas e permite testar eventos e transições sem reutilizar os mocks de apresentação.
  - Validação: Rodar a suíte base em ambiente limpo e confirmar que uma falha de fixture aponta para a causa correta.

## Fase 2 — Base de domínio, persistência e bridge

- [x] TASK 07 — Estruturar o runtime de serviços Rust
  - Objetivo: Criar o ciclo de vida dos serviços de domínio no processo Tauri.
  - Escopo: Estado compartilhado, inicialização ordenada, encerramento, propagação de erros e fronteiras entre domínio, efeitos e bridge.
  - Dependências: TASK 03, TASK 04 e TASK 05.
  - Critérios de conclusão: O runtime compartilhado e testável suporta serviços inativos ou ainda não configurados sem fingir que há networking funcional; inicialização e encerramento têm ordem e propagação de erros definidas.
  - Validação: Testes offline cobrem inicialização parcial, serviço inativo, falha de serviço, cleanup reverso, encerramento e transições inválidas; `greet` e o build da bridge continuam válidos.

- [x] TASK 08 — Implementar persistência local e migrações
  - Objetivo: Tirar o estado confiável da memória sem criar dependência de cloud.
  - Escopo: Inicialização do armazenamento, migrações, leitura/escrita transacional, recuperação de falha e APIs para os modelos definidos.
  - Dependências: TASK 04, TASK 06 e TASK 07.
  - Critérios de conclusão: O storage Rust abre em instalação nova, aplica migrations forward-only com checksum, preserva metadados válidos e falha de modo recuperável sem expor SQL ou conteúdo.
  - Validação: Testes de migration/rollback, reinício, corrupção, versão futura, integridade, runtime e limpeza explícita dos dados locais.

- [x] TASK 09 — Implementar comandos e eventos tipados da bridge
  - Objetivo: Conectar o contrato da TASK 05 ao runtime Rust sem acoplar a UI às implementações internas.
  - Escopo: Registro de comandos, emissão de eventos, desserialização, erros de domínio, validação de origem e lifecycle dos listeners.
  - Dependências: TASK 05, TASK 06 e TASK 07.
  - Critérios de conclusão: A bridge transporta estados e eventos de infraestrutura tipados, trata erro e desconexão, e mantém o fallback web explicitamente demonstrativo; não há ainda estado de produto.
  - Validação: Testes de contrato Rust/Vue, compilação do shell e smoke test da aplicação Tauri.

- [ ] TASK 10 — Integrar modelos reais ao estado do Vue
  - Objetivo: Substituir o acesso direto a dados mockados por stores e adaptadores baseados nos contratos da bridge.
  - Escopo: Estados de carregamento/erro/vazio/stale, sincronização por eventos, seleção de dispositivo, boundary de fixtures e remoção de estados que insinuem operação real.
  - Dependências: TASK 01, TASK 05 e TASK 09.
  - Critérios de conclusão: A UI usa uma fonte de estado explícita; fixtures só aparecem em modo de desenvolvimento e são rotuladas como tal.
  - Validação: Testes de stores e navegação com respostas vazias, atrasadas, duplicadas e com erro.

## Fase 3 — Discovery e presença de dispositivos

- [ ] TASK 11 — Implementar discovery local de candidatos
  - Objetivo: Encontrar dispositivos anunciados na rede local sem conceder confiança automaticamente.
  - Escopo: Anúncio e consulta, identificação inicial, escopo de rede, expiração de candidatos e isolamento entre descoberta e pairing.
  - Dependências: TASK 02, TASK 07 e TASK 09.
  - Critérios de conclusão: Candidatos aparecem e desaparecem conforme o ciclo de discovery, sem serem tratados como pareados ou autorizados.
  - Validação: Teste com peers falsos em rede local e casos de anúncio inválido, duplicado, expirado e fora do escopo.

- [ ] TASK 12 — Implementar presença, heartbeat e reconexão
  - Objetivo: Representar com precisão se um dispositivo descoberto ou conhecido está disponível.
  - Escopo: Estados online/offline/stale, heartbeat, timeout, reconexão, mudanças de endereço e eventos de presença.
  - Dependências: TASK 02, TASK 07, TASK 09 e TASK 11.
  - Critérios de conclusão: A presença não depende de um booleano mockado e não transforma ausência de heartbeat em confiança perdida.
  - Validação: Testes com relógio controlado, perda de rede, retomada e múltiplos anúncios do mesmo dispositivo.

- [ ] TASK 13 — Criar registro de dispositivos conhecidos
  - Objetivo: Separar candidatos descobertos de dispositivos já vistos, pareados ou revogados.
  - Escopo: Registro, metadados, última presença, plataforma, endereço transitório, relação com trust e persistência do estado permitido.
  - Dependências: TASK 03, TASK 08, TASK 11 e TASK 12.
  - Critérios de conclusão: O registro sobrevive a reinício, respeita revogação e não confunde `lastSeen` com dispositivo online.
  - Validação: Testes de hidratação, atualização de metadados, expiração e conflito de identidade.

- [ ] TASK 14 — Entregar a UI real de dispositivos e presença
  - Objetivo: Transformar sidebar, Início e Visão geral em uma leitura honesta do estado dos dispositivos.
  - Escopo: Lista descoberta/conhecida, estados online/offline/stale, atualização, ausência de dispositivos, detalhes básicos e remoção de copy de fundação onde houver dados reais.
  - Dependências: TASK 10, TASK 12 e TASK 13.
  - Critérios de conclusão: A UI exibe origem do estado, tempo de presença, erros recuperáveis e não apresenta candidatos como confiáveis.
  - Validação: Smoke test das rotas em desktop, aproximadamente `680px` e `390px`, incluindo foco, teclado e estados vazios.

## Fase 4 — Pairing, trust e capabilities

- [ ] TASK 15 — Implementar identidade local do dispositivo
  - Objetivo: Dar ao Pulse uma identidade estável para autenticação e relações de confiança.
  - Escopo: Geração, armazenamento, carregamento, rotação planejada, identificação pública e tratamento de perda ou corrupção da identidade.
  - Dependências: TASK 03, TASK 08 e TASK 13.
  - Critérios de conclusão: A identidade não usa nome, IP ou outro dado transitório como prova e seus segredos não ficam expostos à UI.
  - Validação: Testes de primeiro uso, reinício, armazenamento indisponível e identidade inconsistente.

- [ ] TASK 16 — Implementar sessão explícita de pairing
  - Objetivo: Permitir que dois dispositivos confirmem uma relação de confiança por ação explícita.
  - Escopo: Solicitação, exibição de identidade verificável, confirmação nos lados envolvidos, expiração, recusa e cancelamento.
  - Dependências: TASK 03, TASK 09, TASK 11, TASK 13 e TASK 15.
  - Critérios de conclusão: Discovery sozinho nunca pareia; cada resultado de pairing é inequívoco e auditável.
  - Validação: Testes de aprovação, recusa, timeout, peer errado, pedido duplicado e peer offline.

- [ ] TASK 17 — Implementar ciclo de vida de trust e revogação
  - Objetivo: Permitir confiar, desconfiar e revogar um dispositivo de forma previsível.
  - Escopo: Estados de trust, revogação local/remota, invalidação de sessões, re-pairing e distinção entre offline e revogado.
  - Dependências: TASK 08, TASK 15 e TASK 16.
  - Critérios de conclusão: Revogar impede novas operações autorizadas e deixa o motivo/resultado visível sem apagar evidências necessárias.
  - Validação: Testes após revogação, reativação por novo pairing e reinício do app.

- [ ] TASK 18 — Implementar política de capabilities
  - Objetivo: Aplicar autorização por recurso e direção, separada do estado de pairing.
  - Escopo: `available`, `requested`, `granted`, `denied` e `revoked`; capabilities de arquivos, texto/links, Clipboard, notificações, mídia e comandos.
  - Dependências: TASK 03, TASK 08, TASK 16 e TASK 17.
  - Critérios de conclusão: Toda operação futura consulta uma capability; decisões são persistidas, revogáveis e vinculadas ao dispositivo correto.
  - Validação: Matriz de autorização cobrindo cada recurso, direção, revogação e capability ausente.

- [ ] TASK 19 — Entregar UI de pairing, trust e capabilities
  - Objetivo: Dar ao usuário controle compreensível sobre identidade, confiança e permissões.
  - Escopo: Pedidos de pairing, confirmação, lista de dispositivos confiáveis, aprovação por capability, revogação, estados de erro e cópia acessível.
  - Dependências: TASK 14, TASK 16, TASK 17 e TASK 18.
  - Critérios de conclusão: A UI mostra origem, destino, capability, consequência e resultado antes de pedir confirmação; nenhum sucesso é simulado.
  - Validação: Teste de teclado, foco, leitor de tela básico, telas estreitas e todos os estados de aprovação/recusa.

## Fase 5 — Protocolo e segurança de comunicação

- [ ] TASK 20 — Implementar canal seguro entre peers confiáveis
  - Objetivo: Estabelecer comunicação autenticada e protegida depois do pairing.
  - Escopo: Handshake, autenticação de identidade, chaves de sessão, encerramento, expiração e falhas de negociação.
  - Dependências: TASK 02, TASK 03, TASK 15, TASK 16 e TASK 17.
  - Critérios de conclusão: Um peer não pareado, revogado ou com identidade inválida não cria sessão de recurso.
  - Validação: Testes de handshake correto, identidade trocada, replay, downgrade, timeout e encerramento abrupto.

- [ ] TASK 21 — Implementar envelope de mensagens e negociação de capabilities
  - Objetivo: Definir mensagens versionadas para recursos sem acoplar serviços ao transporte escolhido.
  - Escopo: Envelope, versão, correlação, origem/destino, resposta/erro, negociação de capabilities e compatibilidade entre versões.
  - Dependências: TASK 01, TASK 05, TASK 18 e TASK 20.
  - Critérios de conclusão: Serviços conseguem trocar mensagens versionadas e rejeitam recurso ou versão não suportados de forma explícita.
  - Validação: Testes de serialização, mensagens incompletas, versão desconhecida, resposta fora de ordem e capability negada.

- [ ] TASK 22 — Aplicar validação, limites e proteção contra abuso
  - Objetivo: Tornar o protocolo seguro contra entradas malformadas e uso excessivo antes de conectar recursos.
  - Escopo: Tamanho e frequência de mensagens, replay, timeouts, backpressure, erros não reveladores e encerramento de sessão abusiva.
  - Dependências: TASK 03, TASK 06, TASK 20 e TASK 21.
  - Critérios de conclusão: Os limites são centralizados, observáveis e testáveis; nenhuma mensagem vira efeito local sem validação e autorização.
  - Validação: Testes negativos, carga controlada, mensagens truncadas/duplicadas e revisão do threat model.

## Fase 6 — Transferências e compartilhamento de conteúdo

- [ ] TASK 23 — Integrar seleção de arquivos/pastas e política de caminhos Linux
  - Objetivo: Permitir escolher conteúdo local sem expor caminhos indevidos ou depender de input inseguro.
  - Escopo: Picker Tauri, arquivos e pastas, normalização de caminho, permissões, links simbólicos, limites e diretório de destino.
  - Dependências: TASK 03, TASK 05, TASK 07 e TASK 18.
  - Critérios de conclusão: A seleção produz metadados seguros e a UI explica cancelamento, acesso negado e conteúdo fora da política.
  - Validação: Testes com arquivo, pasta, caminho inválido, symlink, permissão negada, nome Unicode e cancelamento.

- [ ] TASK 24 — Implementar o núcleo de sessões de transferência
  - Objetivo: Representar uma transferência real de ponta a ponta no domínio.
  - Escopo: Manifesto, origem/destino, itens, tamanho, estado, progresso, sessão, erro e resultado; sem declarar conclusão antes da confirmação correta.
  - Dependências: TASK 01, TASK 18, TASK 21, TASK 22 e TASK 23.
  - Critérios de conclusão: Uma sessão pode ser criada, observada e finalizada com sucesso ou erro sem dados mockados.
  - Validação: Testes de manifesto, arquivo vazio, pasta, múltiplos itens, erro de transporte e conclusão parcial.

- [ ] TASK 25 — Implementar envio de arquivos e pastas
  - Objetivo: Transferir conteúdo selecionado para um dispositivo confiável.
  - Escopo: Leitura controlada, manifesto, envio em partes, progresso, confirmação do destino e limites de tamanho/tipo.
  - Dependências: TASK 20, TASK 21, TASK 22, TASK 23 e TASK 24.
  - Critérios de conclusão: Arquivos e pastas chegam íntegros ao destino autorizado, com erro recuperável e sem atravessar o diretório permitido.
  - Validação: Testes de integridade, múltiplos itens, arquivo grande, rede interrompida e capability ausente.

- [ ] TASK 26 — Implementar recebimento e resolução de conflitos
  - Objetivo: Controlar onde e como conteúdo recebido será materializado no Linux.
  - Escopo: Aprovação de recebimento, diretório de destino, nomes conflitantes, arquivo parcial, cancelamento e limpeza segura.
  - Dependências: TASK 18, TASK 23, TASK 24 e TASK 25.
  - Critérios de conclusão: O recebimento nunca sobrescreve silenciosamente, não deixa arquivo parcial considerado concluído e informa o resultado.
  - Validação: Testes de conflito, falta de espaço, interrupção, cancelamento, permissão negada e retomada de conteúdo parcial.

- [ ] TASK 27 — Implementar fila, cancelamento, pausa, retomada e recuperação
  - Objetivo: Tornar sessões de transferência operáveis em condições reais de rede e usuário.
  - Escopo: Ordenação da fila, limites de concorrência, cancelamento, pausa/retomada, retry, recuperação após reinício e eventos de progresso.
  - Dependências: TASK 08, TASK 09, TASK 24, TASK 25 e TASK 26.
  - Critérios de conclusão: Cada ação produz um estado legítimo e persistível; a retomada não duplica nem corrompe conteúdo.
  - Validação: Testes de máquina reiniciada, rede perdida, item falho na fila, pausa prolongada e cancelamento em cada etapa.

- [ ] TASK 28 — Entregar a UI real de arquivos e transferências
  - Objetivo: Substituir as telas demonstrativas por fluxos de seleção, confirmação e acompanhamento reais.
  - Escopo: Aba Arquivos, enviar conteúdo, fila global, progresso, pausa, retomada, cancelamento, retry, recebimento, conflito e estados vazios.
  - Dependências: TASK 10, TASK 19, TASK 23, TASK 26 e TASK 27.
  - Critérios de conclusão: Início e Transferências refletem eventos reais e distinguem queued, active, paused, failed, canceled e complete com texto.
  - Validação: Smoke test das cinco rotas de dispositivo, navegação por teclado e responsividade em desktop, `680px` e `390px`.

- [ ] TASK 29 — Adicionar envio explícito de texto e links
  - Objetivo: Compartilhar conteúdo leve usando o mesmo modelo de autorização e acompanhamento.
  - Escopo: Composição, detecção segura de link, limites, confirmação de destino, capability própria e registro do resultado.
  - Dependências: TASK 18, TASK 21, TASK 24 e TASK 28.
  - Critérios de conclusão: Texto e links são enviados sem misturar seu estado com arquivo e sem ativar sincronização contínua por padrão.
  - Validação: Testes de texto vazio/grande, link malformado, caracteres Unicode, capability negada e peer offline.

## Fase 7 — Clipboard, histórico e notificações

- [ ] TASK 30 — Implementar histórico de eventos persistente
  - Objetivo: Registrar decisões e resultados úteis para o usuário sem transformar conteúdo sensível em log indiscriminado.
  - Escopo: Eventos de pairing/trust, presença relevante, transferências, Clipboard, mídia, comandos e notificações; retenção, consulta, paginação e exclusão.
  - Dependências: TASK 04, TASK 08, TASK 18, TASK 24 e TASK 29.
  - Critérios de conclusão: Eventos são persistidos com origem, destino, capacidade, resultado e horário; conteúdo sensível segue a política definida.
  - Validação: Testes de reinício, retenção, exclusão, ordenação, evento duplicado e falha de armazenamento.

- [ ] TASK 31 — Entregar a UI real de histórico
  - Objetivo: Dar leitura e controle sobre eventos persistidos do Pulse.
  - Escopo: Lista, filtros, detalhes, estados de sucesso/erro/cancelamento, paginação, limpeza e indicação de dados indisponíveis.
  - Dependências: TASK 10 e TASK 30.
  - Critérios de conclusão: A rota Histórico deixa de ser vazia por definição e nunca apresenta registro apenas de um mock visual.
  - Validação: Testar histórico vazio, grande, filtrado, com erro de leitura e com itens sensíveis omitidos.

- [ ] TASK 32 — Implementar serviço de notificações locais
  - Objetivo: Avisar o usuário sobre eventos relevantes sem exigir que a janela esteja em primeiro plano.
  - Escopo: Integração Linux/Tauri, pedidos de pairing, conclusão/falha de transferência, revogação, erros importantes e deduplicação.
  - Dependências: TASK 05, TASK 18, TASK 27 e TASK 30.
  - Critérios de conclusão: Notificações derivam de eventos de domínio, respeitam autorização e não anunciam sucesso inexistente.
  - Validação: Testes de permissão negada, janela fechada, evento duplicado, falha da integração e ação sobre a notificação quando aplicável.

- [ ] TASK 33 — Adicionar preferências e estados visíveis de notificação
  - Objetivo: Permitir controle compreensível sobre quais avisos o Pulse mostra.
  - Escopo: Preferências persistentes, silenciamento, categorias, indicação de notificação indisponível e feedback dentro da UI.
  - Dependências: TASK 08, TASK 30 e TASK 32.
  - Critérios de conclusão: O usuário consegue alterar preferências sem perder eventos no histórico e cada estado é explicado por texto.
  - Validação: Testar primeiro uso, reinício, preferência por categoria, silenciamento e recuperação de erro.

- [ ] TASK 34 — Integrar Clipboard local no Linux
  - Objetivo: Ler e escrever Clipboard local somente dentro de uma política explícita.
  - Escopo: Adapter Linux/Tauri, texto e links, alterações locais, limites de tamanho, falhas e ausência de sessão gráfica.
  - Dependências: TASK 03, TASK 05, TASK 07 e TASK 18.
  - Critérios de conclusão: O app distingue Clipboard local de remoto e nunca inicia sincronização contínua sem decisão do usuário.
  - Validação: Testes de leitura, escrita, conteúdo vazio/grande, caracteres Unicode, sessão indisponível e erro de permissão.

- [ ] TASK 35 — Implementar protocolo e políticas de Clipboard
  - Objetivo: Compartilhar Clipboard entre dispositivos conforme capability e política escolhidas.
  - Escopo: Envio explícito, sincronização opcional, origem/horário, deduplicação, retenção, prevenção de loop e revogação.
  - Dependências: TASK 18, TASK 21, TASK 22, TASK 30 e TASK 34.
  - Critérios de conclusão: Cada leitura/escrita remota é autorizada, observável e interrompível; conteúdo não permitido é rejeitado.
  - Validação: Testes de loop entre peers, conteúdo repetido, peer revogado, limite, rede perdida e política desativada.

- [ ] TASK 36 — Entregar a UI real de Clipboard
  - Objetivo: Substituir a rota reservada por uma experiência clara de envio e política do Clipboard.
  - Escopo: Estado local/remoto, envio, sincronização opcional, capability, origem, data, limite, erro e desativação.
  - Dependências: TASK 10, TASK 19, TASK 34 e TASK 35.
  - Critérios de conclusão: A aba Clipboard não confunde preview com conteúdo remoto e preserva feedback textual em todos os estados.
  - Validação: Testar conteúdo vazio, longo, link, peer offline, capability revogada e tela estreita.

## Fase 8 — Mídia e controle remoto

- [ ] TASK 37 — Integrar estado de mídia Linux via MPRIS
  - Objetivo: Observar o estado de players locais por uma integração Linux delimitada.
  - Escopo: Descoberta de player, título/artista, reprodução, posição, volume quando disponível, ausência de player e lifecycle D-Bus.
  - Dependências: TASK 03, TASK 05 e TASK 07.
  - Critérios de conclusão: O adapter expõe apenas ações e dados suportados pelo player, com falha explícita quando o recurso não existe.
  - Validação: Testes com player presente/ausente, múltiplos players, D-Bus indisponível e mudança de estado.

- [ ] TASK 38 — Implementar mídia remota e capability de controle
  - Objetivo: Expor estado e ações de mídia entre dispositivos autorizados.
  - Escopo: Mensagens de estado, comandos suportados, capability `media.read`/`media.control`, confirmação, timeout e divergência de estado.
  - Dependências: TASK 18, TASK 21, TASK 22 e TASK 37.
  - Critérios de conclusão: Controle remoto é negado por padrão, limitado ao contrato e não afirma que uma ação ocorreu sem confirmação local.
  - Validação: Testes de leitura sem controle, ação negada, player trocado, timeout e peer revogado.

- [ ] TASK 39 — Entregar a UI real de mídia
  - Objetivo: Substituir o placeholder de Mídia por leitura e controles autorizados.
  - Escopo: Player ativo, estado, controles suportados, capability, loading/stale/offline, erro e ausência de player.
  - Dependências: TASK 10, TASK 19, TASK 37 e TASK 38.
  - Critérios de conclusão: A UI só mostra controles disponíveis e explica claramente quando leitura ou controle não foram autorizados.
  - Validação: Testar teclado, foco, player ausente, estado stale, capability negada e larguras previstas no design.

- [ ] TASK 40 — Implementar comandos remotos limitados e auditáveis
  - Objetivo: Oferecer ações remotas úteis sem criar execução arbitrária de shell.
  - Escopo: Catálogo allowlisted de ações, parâmetros validados, aprovação por capability, confirmação, timeout, cancelamento, auditoria e revogação.
  - Dependências: TASK 03, TASK 18, TASK 21, TASK 22 e TASK 30.
  - Critérios de conclusão: Só ações pré-definidas e autorizadas são executáveis; cada tentativa tem resultado auditável e nenhum input vira comando livre.
  - Validação: Testes de parâmetro inválido, ação não permitida, timeout, cancelamento, peer revogado e falha do sistema.

- [ ] TASK 41 — Entregar a UI real de controle remoto
  - Objetivo: Substituir a rota reservada por ações remotas compreensíveis e seguras.
  - Escopo: Catálogo de ações, confirmação de consequência, capability, loading, resultado, erro, cancelamento e link para auditoria.
  - Dependências: TASK 10, TASK 19, TASK 30 e TASK 40.
  - Critérios de conclusão: A UI não oferece comandos arbitrários, deixa claro o dispositivo afetado e mostra o resultado textual da ação.
  - Validação: Testar navegação por teclado, confirmação/recusa, falha, peer offline, capability revogada e responsividade.

## Fase 9 — Testes de produto e integração

- [ ] TASK 42 — Cobrir modelos, transições e persistência
  - Objetivo: Garantir que estados críticos não regressem para comportamentos implícitos ou mockados.
  - Escopo: Modelos de dispositivo/trust/capability, transferências, Clipboard, histórico, migrações, retenção e recuperação.
  - Dependências: TASK 08, TASK 18, TASK 27, TASK 30 e TASK 35.
  - Critérios de conclusão: As transições válidas e inválidas têm cobertura e os dados persistidos podem ser reidratados sem perda silenciosa.
  - Validação: Rodar testes unitários com cobertura mínima acordada e revisar casos de erro mais importantes.

- [ ] TASK 43 — Cobrir discovery, presença e pairing em integração
  - Objetivo: Validar o ciclo de vida de peers em uma rede local controlada.
  - Escopo: Peers falsos, anúncios, heartbeat, timeout, identidade, pairing, trust, revogação e re-pairing.
  - Dependências: TASK 11, TASK 12, TASK 16 e TASK 17.
  - Critérios de conclusão: Cenários de rede instável e peers maliciosos básicos são reproduzíveis sem depender da rede do desenvolvedor.
  - Validação: Suite de integração com pelo menos dois peers isolados e execução repetível.

- [ ] TASK 44 — Cobrir protocolo e segurança
  - Objetivo: Detectar regressões nos limites que protegem comunicação e autorização.
  - Escopo: Handshake, versionamento, serialização, replay, downgrade, payload malformado, limites, capabilities e encerramento.
  - Dependências: TASK 20, TASK 21 e TASK 22.
  - Critérios de conclusão: Casos negativos são parte permanente da suíte e falhas não expõem segredos nem autorizam efeitos.
  - Validação: Testes de contrato, propriedades/fuzzing onde fizer sentido e revisão dos resultados contra o threat model.

- [ ] TASK 45 — Cobrir serviços de recursos
  - Objetivo: Validar transferências, Clipboard, notificações, mídia e comandos sem depender da UI.
  - Escopo: Sessões, arquivos/pastas, conflitos, pausa/retomada, Clipboard, eventos, MPRIS, allowlist e auditoria.
  - Dependências: TASK 25, TASK 26, TASK 32, TASK 35, TASK 38 e TASK 40.
  - Critérios de conclusão: Cada serviço testa sucesso, erro, cancelamento, timeout, capability ausente e peer offline.
  - Validação: Suite de serviços com adapters Linux falsos e fixtures de falha.

- [ ] TASK 46 — Cobrir Vue, stores, bridge e estados de UI
  - Objetivo: Garantir que a interface represente corretamente eventos e estados de domínio.
  - Escopo: Componentes, stores, rotas, eventos duplicados/atrasados, loading, vazio, stale, erro, confirmação e acessibilidade básica.
  - Dependências: TASK 10, TASK 19, TASK 28, TASK 31, TASK 36, TASK 39 e TASK 41.
  - Critérios de conclusão: A UI não depende de timers ou mocks implícitos para declarar sucesso e mantém nomes/estados acessíveis.
  - Validação: Testes de componentes e stores, mais smoke test das rotas em desktop, `680px` e `390px`.

- [ ] TASK 47 — Criar smoke tests Tauri/Linux e matriz de aceitação
  - Objetivo: Validar o produto empacotado como aplicativo desktop, não apenas como prévia web.
  - Escopo: Inicialização Tauri, capabilities, bridge, picker, Clipboard, notificações, D-Bus/MPRIS, dois peers e fluxos críticos de transferência.
  - Dependências: TASK 23, TASK 28, TASK 32, TASK 36, TASK 39, TASK 41 e TASK 46.
  - Critérios de conclusão: Existe uma matriz de ambientes, pré-condições e resultados esperados para os fluxos críticos.
  - Validação: Execução automatizada quando possível e checklist manual explícito para integrações dependentes do desktop Linux.

## Fase 10 — Segurança operacional e distribuição

- [ ] TASK 48 — Revisar capabilities Tauri, CSP e permissões Linux
  - Objetivo: Reduzir a superfície de ataque antes de distribuir o aplicativo.
  - Escopo: Capabilities mínimas por recurso, CSP, escopos de filesystem, rede, notificações, Clipboard, D-Bus, controle de processos e segredos.
  - Dependências: TASK 03, TASK 23, TASK 32, TASK 34, TASK 37, TASK 40 e TASK 47.
  - Critérios de conclusão: Nenhuma permissão ampla permanece sem justificativa; a configuração deixa de depender de `csp: null` e documenta exceções necessárias.
  - Validação: Auditoria de permissões, testes com cada capability ausente e inspeção do bundle/configuração final.

- [ ] TASK 49 — Habilitar e validar bundles Linux
  - Objetivo: Transformar o shell executável em um artefato instalável.
  - Escopo: Ativar bundle, ícones, metadados, formatos Linux escolhidos, dependências, instalação, desinstalação e diretórios de dados.
  - Dependências: TASK 47 e TASK 48.
  - Critérios de conclusão: Os artefatos instalados iniciam, encontram seus recursos e respeitam o ciclo de vida dos dados locais.
  - Validação: Build limpo e instalação em ambientes Linux suportados, incluindo upgrade sobre uma versão anterior.

- [ ] TASK 50 — Automatizar CI, artefatos e verificações de release
  - Objetivo: Tornar builds, testes e artefatos reproduzíveis antes de publicar uma versão.
  - Escopo: Pipeline para typecheck/build/Rust/tests, matriz Linux suportada, cache controlado, checks de bundle, versionamento e armazenamento de artefatos.
  - Dependências: TASK 42, TASK 43, TASK 44, TASK 45, TASK 46, TASK 47 e TASK 49.
  - Critérios de conclusão: Uma mudança só pode avançar com validações definidas e os artefatos têm versão e origem identificáveis.
  - Validação: Executar pipeline em branch de teste, baixar artefatos e repetir instalação a partir deles.

- [ ] TASK 51 — Fechar processo de lançamento e operação
  - Objetivo: Definir como o Pulse será entregue, atualizado, diagnosticado e removido com segurança.
  - Escopo: Notas de versão, compatibilidade de migrações, suporte, coleta de diagnóstico sem dados privados, assinatura/verificação quando aplicável e procedimento de rollback.
  - Dependências: TASK 04, TASK 30, TASK 48, TASK 49 e TASK 50.
  - Critérios de conclusão: Existe um processo de release que explica instalação, upgrade, remoção, recuperação e limites de suporte sem prometer capacidades não entregues.
  - Validação: Simulação de release, upgrade com dados persistidos, rollback documentado e revisão final dos documentos raiz.

## Próxima task recomendada

**TASK 10 — Integrar modelos reais ao estado do Vue.** A bridge de infraestrutura agora está tipada e redigida; a próxima etapa deve hidratar stores com snapshot/eventos, mantendo mocks rotulados e estados de loading, erro, vazio, stale e offline honestos.
