# Trevia Editor Engine

Uma engine de editor de texto de alta performance com suporte a paginação, construída com WebAssembly e Rust.

## Características

- **Engine WebAssembly**: Core em Rust para máxima performance
- **Paginação Automática**: Distribuição inteligente de texto entre páginas
- **Modelo Unificado**: Estado compartilhado entre todas as páginas
- **Renderização Canvas**: Controle total sobre renderização
- **Sincronização Perfeita**: Eliminação de dessincronização entre páginas

## Arquitetura

```
[Document Model (WASM)]
        ↓
[Layout Engine (WASM)]
        ↓
[Canvas Renderer (JS)]
        ↓
[Multiple Pages (Synchronized)]
```

### Componentes Principais

1. **DocumentModel**: Modelo unificado de documento em memória
2. **LayoutEngine**: Cálculos de layout e quebras de página
3. **TreviaEditor**: Interface principal da engine
4. **Canvas Renderer**: Renderização final em JavaScript

## Instalação

### Pré-requisitos

- Rust (última versão estável)
- wasm-pack
- Node.js (para desenvolvimento)

### Build

```bash
# Instalar wasm-pack se não tiver
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# Build da engine
npm run build

# Executar demo
npm run serve
```

## Uso

### Inicialização Básica

```javascript
import init, { TreviaEditor } from './pkg/trevia_editor_engine.js';

await init();

const editor = new TreviaEditor(800, 1000); // largura, altura da página
editor.set_margins(50, 50, 50, 50); // top, right, bottom, left
```

### Operações de Texto

```javascript
// Inserir texto
editor.insert_text("Olá mundo!");

// Deletar texto
editor.delete_text(5); // deleta 5 caracteres antes do cursor

// Posicionar cursor
editor.set_cursor_position(10);

// Seleção
editor.set_selection(0, 10);
editor.delete_selection();
```

### Layout e Renderização

```javascript
// Recomputar layout
editor.recompute_layout();

// Obter informações de layout
const pageCount = editor.get_page_count();
const pageLayout = editor.get_page_layout(0);

// Cursor na tela
const cursorPos = editor.get_cursor_screen_position();
```

## Solução para o Problema de Composição

### Problema Original
- Apenas primeira página editável
- Páginas adicionais ficam "inativas"
- Dessincronização entre páginas

### Solução Implementada

1. **Document Model Unificado**: Todo o texto fica em uma única estrutura
2. **Virtual Pages**: Páginas são views calculadas do documento
3. **Event Delegation**: Eventos capturados globalmente
4. **Automatic Reflow**: Mudanças propagam para todas as páginas

### Fluxo de Sincronização

```rust
// Em Rust/WASM
fn handle_edit(&mut self, position: usize, operation: EditOperation) {
    // 1. Atualiza modelo unificado
    self.document.apply_operation(operation);
    
    // 2. Recalcula layout afetado
    let affected_pages = self.layout_engine.reflow_from(position);
    
    // 3. Notifica JavaScript para re-render
    notify_pages_changed(affected_pages);
}
```

## Performance

- **WebAssembly**: Cálculos de layout 3x+ mais rápidos
- **Virtual Pagination**: Suporta documentos de qualquer tamanho
- **Canvas Rendering**: Renderização otimizada pelo browser
- **Memory Efficient**: Modelo unificado reduz uso de memória

## Estrutura do Projeto

```
src/
├── lib.rs          # Ponto de entrada
├── types.rs        # Tipos básicos (Position, Size, etc.)
├── document.rs     # Modelo de documento
├── layout.rs       # Engine de layout e paginação
└── editor.rs       # Interface principal

pkg/                # Output do wasm-pack
index.html          # Demo interativo
```

## Desenvolvimento

### Comandos Úteis

```bash
# Build para web
npm run build

# Build para Node.js
npm run build:node

# Desenvolvimento com hot reload
npm run dev

# Testes
npm test
```

### Debug

O demo inclui modo debug que mostra:
- Informações de layout em tempo real
- Estados internos da engine
- Métricas de performance

## Roadmap

- [ ] Suporte a estilos de texto (negrito, itálico, etc.)
- [ ] Undo/Redo system
- [ ] Colaboração em tempo real
- [ ] Export para PDF/DOCX
- [ ] Plugin system
- [ ] Mobile support

## Contribuição

1. Fork o projeto
2. Crie uma branch para sua feature
3. Commit suas mudanças
4. Push para a branch
5. Abra um Pull Request

## Licença

MIT License - veja o arquivo LICENSE para detalhes.