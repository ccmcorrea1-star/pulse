---
name: Pulse
description: Central de controle local para transferências entre dispositivos confiáveis.
colors:
  canvas: "#080b0e"
  workspace: "#101519"
  surface: "#12181d"
  surface-strong: "#171d22"
  ink: "#f2f5f5"
  muted: "#9aa4ad"
  divider: "#222a31"
  selection: "#1d2329"
  signal-green: "#39d463"
  signal-green-dark: "#1c9e46"
  signal-blue: "#39b9ed"
  signal-yellow: "#e4b74e"
  attention-surface: "#252016"
  transfer-surface: "#121b22"
  focus: "#6be38a"
typography:
  display:
    fontFamily: 'Inter, ui-sans-serif, system-ui, -apple-system, "Segoe UI", sans-serif'
    fontSize: "28px"
    fontWeight: 600
    lineHeight: 1.1
    letterSpacing: "-0.04em"
  headline:
    fontFamily: 'Inter, ui-sans-serif, system-ui, -apple-system, "Segoe UI", sans-serif'
    fontSize: "20px"
    fontWeight: 600
    lineHeight: 1.2
    letterSpacing: "-0.02em"
  body:
    fontFamily: 'Inter, ui-sans-serif, system-ui, -apple-system, "Segoe UI", sans-serif'
    fontSize: "14px"
    lineHeight: 1.45
  label:
    fontFamily: 'Inter, ui-sans-serif, system-ui, -apple-system, "Segoe UI", sans-serif'
    fontSize: "12px"
    fontWeight: 600
    letterSpacing: "0.1em"
  mono:
    fontFamily: 'ui-monospace, SFMono-Regular, Menlo, monospace'
    fontSize: "12px"
    fontWeight: 600
    lineHeight: 1.2
    letterSpacing: "0.08em"
rounded:
  control: "8px"
  panel: "10px"
  shell: "16px"
spacing:
  control: "10px"
  section: "16px"
  layout: "20px"
  shell: "29px"
components:
  panel:
    backgroundColor: "{colors.surface}"
    border: "1px solid {colors.divider}"
    rounded: "{rounded.panel}"
  signal-button:
    backgroundColor: "{colors.signal-green}"
    textColor: "#07100a"
    rounded: "{rounded.control}"
  ghost-button:
    backgroundColor: "{colors.surface-strong}"
    textColor: "{colors.ink}"
    border: "1px solid {colors.divider}"
    rounded: "{rounded.control}"
---

# Design System: Pulse

## Overview

**Creative North Star: “Central de comando da rede local”.**

Pulse é um painel desktop escuro para acompanhar conteúdo em movimento entre dispositivos confiáveis. A referência visual é uma estação técnica silenciosa: shell preto-azulado, linhas finas, superfícies quase foscas, tipografia de sistema precisa e verde luminoso reservado para conexão, progresso e ação confirmada.

O primeiro viewport deve provar o mecanismo em uso: um dispositivo selecionado no cabeçalho, navegação lateral persistente, envio rápido, transferência em andamento, atividade recente e os sinais operacionais do dispositivo ao lado. A interface pode ser densa, mas nunca deve parecer um terminal ou uma parede de cards.

### Key characteristics

- Densidade operacional serena: linhas de lista e painéis organizam a leitura sem excesso de decoração.
- Verde como sinal vivo: conexão, status pronto, progresso e foco de ação.
- Azul/ciano para conteúdo recebido e superfície de mídia; amarelo somente para atenção pendente.
- Uma janela externa elevada sobre um canvas quase preto; profundidade interna vem de bordas e tons, não de sombras repetidas.
- Copy em português brasileiro, direta e acompanhada de estado textual; cor sozinha nunca carrega significado.

## Colors

### Signal palette

- **Verde Pulse** (`#39d463`): conectado, ativo, concluído, progresso e foco positivo.
- **Azul recebido** (`#39b9ed`): conteúdo recebido, Clipboard e estados de sincronização.
- **Amarelo de atenção** (`#e4b74e`): aprovação pendente, rede interrompida ou decisão necessária.

### Neutral palette

- **Canvas** (`#080b0e`) e **workspace** (`#101519`) formam o campo escuro contínuo.
- **Surface** (`#12181d`) e **surface strong** (`#171d22`) distinguem painéis e controles por tom.
- **Ink** (`#f2f5f5`), **muted** (`#9aa4ad`) e **divider** (`#222a31`) definem uma hierarquia de alto contraste sem branco estourado.

### Named rules

**The Signal Has a Label Rule.** Verde, azul e amarelo só aparecem com texto, valor, ícone ou ação adjacente que explique o estado.

**The Quiet Surface Rule.** O produto deve continuar legível sem brilho, gradiente ou sombra interna; contraste vem de superfície, borda e espaçamento.

## Typography

Inter ou a sans-serif de sistema conduz toda a UI. Nomes de arquivos e títulos usam peso médio; metadados usam tamanho menor e cinza azulado; valores sensíveis e técnicos usam mono apenas quando isso aumenta a leitura.

- Cabeçalho do dispositivo: 28px / 600.
- Títulos de painel: 16–20px / 500–600.
- Corpo: 14–16px / 400.
- Metadados: 12–14px / 400.
- Rótulos de seção e dispositivos: 11–12px, espaçamento aberto.

Não usar fontes display decorativas, texto em caixa alta como padrão ou monospace como fantasia técnica.

## Layout

O shell tem duas colunas: rail lateral de aproximadamente 263px e workspace fluido. O rail ocupa toda a altura e concentra marca, navegação, dispositivos, configurações e perfil. O cabeçalho do workspace tem cerca de 126px e mostra o dispositivo selecionado, plataforma, conexão, bateria, rede local e menu de contexto.

Logo abaixo, abas de ferramenta ocupam uma faixa compacta com régua verde no item ativo. O resumo usa grid assimétrico: conteúdo principal fluido à esquerda e coluna lateral de aproximadamente 417px à direita. O conteúdo principal segue a ordem envio rápido → transferência → atividade; a coluna lateral segue mídia → status → ações rápidas.

Em até 1180px, o rail encolhe e o grid perde espaço lateral. Em até 920px, o rail vira uma faixa horizontal, os dispositivos ficam roláveis e o conteúdo passa para uma coluna. Em até 620px, controles e painéis empilham; a faixa de abas e os dispositivos mantêm rolagem horizontal sem provocar overflow no documento.

## Elevation & depth

Somente o shell externo tem sombra persistente (`0 26px 90px rgba(0,0,0,.56)`). Painéis internos usam borda de 1px e diferença tonal. O foco usa halo verde sutil; o dropzone ativo troca borda e fundo juntos. Não aplicar sombra individual em cada painel.

## Components

### Navigation

O item ativo do rail recebe fundo `#1c2227` e uma régua verde de 2px. Abas internas usam fundo escuro e uma linha verde inferior; não usar pílulas para navegação principal.

### Panels

Painéis têm fundo `#12181d`, borda `#222a31` e raio de 10px. Listas internas usam linhas horizontais. O produto aceita painéis porque a referência é uma estação de trabalho, mas cada painel deve conter uma unidade operacional clara, nunca apenas um título e uma decoração.

### Buttons

Controles têm raio de 8px, borda fina e altura entre 42–48px quando são ações primárias da tela. Verde preenchido é reservado à consequência principal. Botões secundários usam superfície forte e borda neutra; hover pode elevar o contraste da borda ou aplicar verde no texto.

### Dropzone

A área de envio usa borda tracejada verde escura, fundo quase preto e ícone de upload em verde. O estado de arrastar reforça borda, fundo e halo ao mesmo tempo. As quatro ações rápidas abaixo ficam em controles iguais, com ícones semânticos e acentos individuais discretos.

### Transfer and activity

Transferências apresentam tipo de arquivo, nome, metadados, barra de progresso e pausa no mesmo alinhamento. Atividade é uma lista de linhas com ícone circular, título, origem/tamanho e horário. Verde comunica envio/conclusão; azul comunica recebimento.

### Media and device status

Mídia usa uma capa real, controles circulares simples, barra de progresso e volume. Status do dispositivo é uma lista compacta de bateria, rede, armazenamento e confiança. Ações rápidas são linhas clicáveis com ícone, texto e seta, sem transformar cada ação em um card isolado.

### Responsive and accessibility

Todos os controles conservam foco visível e alvos de toque confortáveis. A responsividade reorganiza contexto, não remove estado essencial. Rede offline sempre aparece como texto e como mudança de sinal; transferência pausada, erro, carregamento, vazio e aprovação permanecem operáveis.

## Do's and Don'ts

### Do

- Use verde apenas quando houver uma ação ou estado positivo para explicar.
- Mantenha bordas de 1px, raios discretos e separadores horizontais.
- Preserve a hierarquia rail → cabeçalho do dispositivo → abas → workspace.
- Use conteúdo demonstrativo honesto e mantenha a rede local como contexto central.
- Teste desktop e mobile, especialmente em torno de 390px.

### Don't

- Não reintroduza a paleta clara de papel quente do protótipo anterior.
- Não use gradientes, glassmorphism, sombras em todos os cards ou brilho neon como decoração.
- Não dependa apenas de cor para dizer online, offline, recebido ou pendente.
- Não transforme todos os itens em pílulas ou cartões iguais.
- Não use emoji ou glifos Unicode como substitutos para ícones em novos componentes; prefira SVGs de traço consistente.
