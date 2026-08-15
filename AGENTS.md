# Guia para agentes

Instruções operacionais para trabalhar neste repositório. Não replique aqui decisões de produto, UI ou arquitetura: consulte os documentos fonte.

## Fontes de decisão

| Assunto | Fonte principal |
| --- | --- |
| Produto, escopo e roadmap funcional | [`PRODUCT.md`](PRODUCT.md) |
| UI/UX, layout, tokens, estados e iconografia | [`DESIGN.md`](DESIGN.md) |
| Arquitetura, bridge Tauri/Rust e módulos futuros | [`SYSTEM-DESIGN.md`](SYSTEM-DESIGN.md) |
| Entrada rápida e comandos principais | [`README.md`](README.md) |

Se a implementação e a documentação divergirem, verifique o código primeiro, corrija a documentação na mesma tarefa quando esse for o objetivo e não invente comportamento para preencher lacunas.

## Estado atual que importa para agentes

- O app atual é Vue 3 + TypeScript + Vite dentro de Tauri 2; o ponto de entrada é `src/main.ts` e o shell Rust está em `src-tauri/src/`.
- Dispositivos e transferências ainda são mocks efêmeros de Pinia. Não há discovery, pairing, rede, persistência, Clipboard, mídia ou transferências reais.
- O único command Rust é `greet`, exposto para testar a bridge em `SettingsView`.
- As rotas de dispositivo já reservam `Visão geral`, `Arquivos`, `Clipboard`, `Mídia` e `Controle`, mas a maior parte delas é placeholder.
- `10-pulse-resumo.html` é um protótipo legado; não é o ponto de entrada do aplicativo atual.

## Comandos

```bash
npm install
npm run dev
npm run typecheck
npm run build
npm run tauri:dev
npm run tauri:build
cargo check --manifest-path src-tauri/Cargo.toml
```

`npm run dev` serve a prévia web na porta `1420`. `npm run tauri:dev` executa o shell desktop e valida a bridge com Rust. `tauri:build` existe no script, mas o bundle está desativado em `src-tauri/tauri.conf.json`.

Após mudanças de UI, confira Início, Transferências, Histórico, Configurações e as cinco abas de dispositivo em desktop e em torno de `680px`/`390px`.

## Convenções de implementação

- Use dois espaços em Vue, TypeScript, CSS e Markdown quando houver indentação.
- Use `<script setup lang="ts">`, aliases `@/*` e componentes pequenos no frontend.
- Prefira classes/token existentes de Tailwind e variáveis de `src/styles/index.css`; classes CSS próprias devem seguir `pulse-` + kebab-case.
- Mantenha copy em português brasileiro, em sentence case, com estado textual e nomes de ação claros.
- Preserve `focus-visible`, landmarks e nomes acessíveis.
- Use Lucide para ícones de interface e Simple Icons somente para marcas/plataformas.
- Mantenha Rust organizado por domínio em `src-tauri/src/` e não crie capabilities Tauri amplas sem necessidade documentada.

## Regras de documentação e segurança

- Use as quatro fontes principais acima e evite duplicar a mesma decisão em mais de um documento.
- Ao documentar uma feature, marque explicitamente se está **implementada**, **estruturada**, **planejada** ou é **futura**.
- Nunca apresente mock, placeholder ou rota como integração funcional.
- Não adicione credenciais, dados privados de rede ou lógica de transporte de produção ao protótipo.
- Não altere `10-pulse-resumo.html` ou assets legados para implementar uma feature do app atual sem uma solicitação específica.
- `.agents/`, `.codex/` e `.impeccable/` são arquivos auxiliares/gerados de tooling; não são fontes de decisão do Pulse.

## Validação antes de entregar

1. Confira `git diff` e confirme que a mudança ficou no escopo pedido.
2. Execute `npm run typecheck`, `npm run build` e `cargo check --manifest-path src-tauri/Cargo.toml` quando o ambiente permitir.
3. Para mudanças de UI, faça smoke test das rotas e da responsividade.
4. Para mudanças de documentação, releia todos os Markdown raiz e procure nomes, status e comandos contraditórios.
