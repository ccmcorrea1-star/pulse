# Pulse — Design de UI/UX

Este é o documento principal para decisões de interface e experiência do Pulse. Ele descreve primeiro o que existe hoje em `src/` e, quando necessário, registra a direção para áreas ainda não implementadas. O roadmap funcional fica no [PRODUCT.md](PRODUCT.md); a arquitetura fica no [SYSTEM-DESIGN.md](SYSTEM-DESIGN.md).

## Direção visual

O Pulse é uma central de comando local: dark, compacta e desktop-first. A interface deve transmitir uma estação operacional calma, com separadores finos, superfícies discretas e sinais de estado claros. O verde identifica ação/estado positivo; azul identifica informação ou transferência; amarelo chama atenção para uma decisão. Nenhuma cor deve carregar significado sozinha.

O copy da interface é em português brasileiro, direto e orientado à ação. Estados mockados devem continuar explicitamente identificados como mock, estrutura inicial ou rota preparada.

## Estrutura implementada

### Shell e navegação

- `AppShell` compõe a aplicação em sidebar + área de conteúdo.
- A sidebar global contém a marca Pulse, `Início`, `Transferências`, `Histórico`, a lista de dispositivos e `Configurações`.
- A lista de dispositivos é alimentada pelo store de apresentação, que identifica se a fonte é fixture de desenvolvimento ou vazio não configurado, e usa a rota `/device/:id` como contexto selecionado.
- A área principal tem cabeçalho com o contexto `rede local / ambiente de base`, o estado `shell pronto` e o acesso às configurações.
- A navegação usa Vue Router; não há abas implementadas no HTML legado como fonte paralela.

### Contexto de dispositivo

Cada dispositivo tem uma página própria com estado online/offline, retorno para o Início e abas horizontais:

`Visão geral` · `Arquivos` · `Clipboard` · `Mídia` · `Controle`

As cinco rotas existem. Hoje, as seções usam `DeviceSectionView`; Mídia também mostra `MediaPlaceholder`. Isso é contrato de navegação e composição, não implementação dessas capacidades.

### Views atuais

| View | Papel atual |
| --- | --- |
| `HomeView` | Apresenta a fundação, as fronteiras futuras e coleções vindas de fixture de desenvolvimento ou de estado vazio não configurado. |
| `TransfersView` | Reserva a fila de transferências; ações de envio e pausa estão desabilitadas. |
| `HistoryView` | Reserva a leitura de histórico; mostra estado vazio sem persistência. |
| `DeviceView` + `DeviceSectionView` | Oferece o contexto do dispositivo e as cinco abas preparadas. |
| `SettingsView` | Expõe o teste da bridge Vue ↔ Rust e os dados da base instalada. |

## Layout e responsividade

- O shell ocupa no mínimo a altura da janela e usa uma sidebar de `252px`.
- A sidebar reduz para `218px` até `920px`.
- A área de conteúdo é fluida, com scroll vertical próprio e padding horizontal de `24px` (`32px` a partir de `sm`, `40px` a partir de `lg`).
- Até `680px`, o shell vira fluxo vertical; a lista de dispositivos da sidebar rola horizontalmente e cada item mantém largura mínima de `170px`.
- O `body` mantém largura mínima de `320px`.
- `useResponsive` já fornece os media queries de `920px` e `680px`, mas o comportamento atual é principalmente controlado pelo CSS e não há lógica de domínio dependente desse composable.

O layout deve reorganizar o conteúdo sem esconder estado importante. Em telas menores, preservar nomes, status, destino e feedback textual.

## Tokens visuais

A fonte de verdade dos tokens é [`src/styles/index.css`](src/styles/index.css). Classes Tailwind usam esses valores por meio de `@theme inline`.

### Cor

| Token CSS | Valor | Uso |
| --- | --- | --- |
| `--pulse-background` | `#080b0e` | Canvas e fundo geral |
| `--pulse-surface` | `#12181d` | Sidebar e painéis |
| `--pulse-surface-hover` | `#171d22` | Hover de superfícies |
| `--pulse-surface-raised` | `#1c2227` | Item ativo e controles elevados |
| `--pulse-border` | `#222a31` | Bordas e divisores |
| `--pulse-foreground` | `#f2f5f5` | Texto principal |
| `--pulse-muted` | `#9aa4ad` | Texto secundário e metadados |
| `--pulse-muted-strong` | `#c1c8cc` | Texto secundário com mais contraste |
| `--pulse-accent` / `--pulse-success` | `#39d463` | Ação e estado positivo |
| `--pulse-accent-strong` | `#6be38a` | Hover e foco positivo |
| `--pulse-warning` | `#e4b74e` | Atenção e fila mockada |
| `--pulse-destructive` | `#ed7567` | Falha ou ação destrutiva futura |
| `--pulse-info` | `#39b9ed` | Informação e transferência |

### Forma, espaçamento e tipografia

- Controle: raio `8px`; painel: `10px`; shell: `16px`.
- Espaçamentos nomeados: controle `10px`, seção `16px`, layout `20px`, shell `29px`.
- Corpo: Inter ou sans-serif de sistema, `14px` com line-height `1.45`.
- Valores técnicos e percentuais podem usar `ui-monospace`; não usar monospace como decoração.
- O foco visível usa outline de `2px`, offset de `3px`, em `--pulse-accent-strong`.
- Sombras, gradientes e brilho não são necessários para criar hierarquia; preferir tom, borda e espaço.

## Componentes e iconografia

### Componentes existentes

- `Button.vue`: variantes `default`, `secondary`, `ghost` e `outline`; tamanhos `default`, `sm` e `icon`.
- `Badge.vue`: variantes `default`, `muted`, `warning` e `info`.
- `BrandMark.vue`: atualmente renderiza o ícone Linux de `simple-icons`.
- `DeviceList.vue` e `TransferPreview.vue`: componentes de composição para os dados demonstrativos.
- `MediaPlaceholder.vue`: estado explícito de área futura de mídia.

### Regras de ícones

- Usar Lucide para ações, navegação e estados de interface.
- Usar Simple Icons apenas para marcas ou plataformas, como o Linux atual da marca/base.
- Não introduzir emoji ou glifos Unicode como substitutos de ícones.
- Ícone deve vir acompanhado de texto ou de um nome acessível quando representar uma ação.
- Não criar uma linguagem de ícones diferente para cada view; manter traço e peso consistentes.

### Estados

Estados já representados na UI:

- fundação inicial / shell pronto;
- fixture de desenvolvimento sem networking;
- bridge em sincronização, estado observado, offline/não configurado ou erro de sincronização;
- dispositivo online ou offline, sempre com texto;
- transferência em andamento ou na fila, sempre com percentual/status textual;
- rota preparada, estrutura inicial e vazio;
- bridge não testada, testando, respondendo ou em falha.

Estados planejados — erro de rede, pedido de pareamento, aprovação, pausa real, retomada, cancelamento e conclusão persistida — devem seguir a mesma regra: texto explicativo, ação recuperável quando aplicável e não dependência exclusiva de cor.

## Acessibilidade e comportamento

- Preservar `focus-visible` em links e botões.
- Manter `aria-label` quando um controle for somente ícone e `aria-label`/landmark em navegação.
- Usar sentence case em português e verbos que descrevam a consequência da ação.
- Não reduzir o status a um ponto colorido; exibir o rótulo online/offline, progresso ou falha.
- Manter alvos de interação confortáveis em desktop e mobile.
- Para novos fluxos de confiança, mostrar origem, destino, capacidade solicitada e resultado antes de pedir confirmação.

## Referências visuais e ativos

- A UI atual está em `src/`, com estilos em [`src/styles/index.css`](src/styles/index.css).
- [`10-pulse-resumo.html`](10-pulse-resumo.html) é um mockup HTML standalone anterior. Pode servir como histórico de exploração, mas sua paleta clara e seu comportamento não definem o app atual.
- [`assets/media-live-performance.png`](assets/media-live-performance.png) é um asset visual disponível no repositório e não está referenciado pelo frontend atual.
- `src-tauri/icons/` contém os ícones do shell Tauri; o bundle está desativado hoje em `tauri.conf.json`.

## Regras de evolução

- Preferir tokens existentes a novas cores ou raios.
- Adicionar um componente quando houver comportamento reutilizável; não transformar cada linha em um card independente.
- Documentar uma decisão aqui quando ela alterar layout, hierarquia, estado, copy ou iconografia.
- Se uma tela ainda for placeholder, mantê-la honesta e não simular sucesso de uma operação que não existe.
