# Pulse

Pulse é a fundação de um app desktop Linux para integração local entre dispositivos, construído com Tauri 2, Vue 3, TypeScript, Vite, Rust, Tailwind CSS v4 e a configuração de componentes shadcn-vue, além de Pinia, Vue Router, VueUse, Lucide e Simple Icons.

O estado atual é uma base navegável `0.1.0`: a UI, o router, os stores Pinia, as rotas de dispositivo e uma bridge Rust mínima estão implementados; dispositivos e transferências são mockados e não há networking, persistência ou transferência real.

## Executar

```bash
npm install
npm run dev        # prévia web em http://localhost:1420
npm run tauri:dev  # shell desktop Tauri
```

Validação rápida:

```bash
npm run typecheck
npm run build
cargo check --manifest-path src-tauri/Cargo.toml
```

## Documentação

- [PRODUCT.md](PRODUCT.md) — produto, escopo atual e roadmap funcional.
- [DESIGN.md](DESIGN.md) — fonte principal da UI/UX e do sistema visual.
- [SYSTEM-DESIGN.md](SYSTEM-DESIGN.md) — fonte principal da arquitetura e da direção técnica.
- [AGENTS.md](AGENTS.md) — instruções para agentes e validação.

`10-pulse-resumo.html` é um protótipo HTML legado, não o ponto de entrada do app atual. O frontend ativo começa em `index.html` e `src/main.ts`.
