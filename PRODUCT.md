# Product

<!-- impeccable:product-schema 1 -->

## Platform

adaptive

## Users

Pessoas que administram os próprios dispositivos ou uma pequena rede doméstica ou de equipe. Elas precisam decidir rapidamente o que pode entrar, sair ou aguardar aprovação entre dispositivos conectados à mesma rede local.

## Product Purpose

Pulse permite compartilhar arquivos e conteúdo de Clipboard entre dispositivos confiáveis na rede local. O objetivo é dar visibilidade e controle rápido sobre transferências, solicitações e conteúdos recentes em desktop e mobile.

## Positioning

Em vez de depender de um serviço remoto para uma troca pontual, o Pulse centraliza a decisão e o acompanhamento de transferências entre dispositivos confiáveis na própria rede local.

## Operating Context

O uso acontece ao alternar entre dispositivos pessoais ou de uma equipe pequena conectados à mesma rede. Os fluxos incluem aprovar ou recusar solicitações, acompanhar e pausar transferências, selecionar um dispositivo, enviar arquivos, textos, links e imagens, e consultar o histórico e o Clipboard recente.

## Capabilities and Constraints

- Aplicativo planejado em Tauri para desktop e mobile.
- O repositório atual é um mockup estático em HTML, CSS e JavaScript, com estado local em memória e dados demonstrativos; não realiza transferências nem se conecta à rede.
- O mockup cobre Resumo, Dispositivos, Histórico e Clipboard, além de seleção de dispositivos, aprovação ou recusa, pausa ou retomada e mensagens de feedback.
- O produto está aberto a mudanças; compromissos adicionais de privacidade, criptografia, marca e acessibilidade ainda não estão definidos.
- Não usar credenciais, dados privados de rede ou lógica de transferência de produção neste protótipo.

## Evidence on Hand

- Mockup funcional: `10-pulse-resumo.html`.
- Documento de arquitetura e comportamento atual: `SYSTEM-DESIGN.md`.
- Não há backend, transporte real, persistência, provas externas, depoimentos ou ativos de marca fornecidos.

## Product Principles

- Dar controle explícito à pessoa antes de movimentar conteúdo entre dispositivos.
- Tornar o estado da rede e das transferências fácil de entender rapidamente.
- Manter os fluxos locais e entre dispositivos confiáveis como contexto central do produto.
- Distinguir claramente comportamento demonstrativo de capacidades reais de produção.
