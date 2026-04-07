# I3 Languageserver

## Features

- Duplicate bindings warnings 

## Installation

  cargo build --release

## AstroNvim configuration

```lua
-- ~/.config/nvim/lsp/i3_lsp.lua
return {
  cmd = { "/home/johndoe/git/i3-lsp/target/debug/i3-lsp" },
  filetypes = { "i3config" },
  single_file_support = true,
}
```

```lua
-- ~/.config/nvim/lua/plugins/lsp.lua
return {
  {
    "AstroNvim/astrolsp",
    opts = {
      servers = { "i3_lsp" },
    },
  },
}
```

```lua
vim.filetype.add({
  pattern = {
    [".*/i3/config"] = "i3config",
    [".*/i3blocks/config"] = "i3config",
    [".*%.i3config"] = "i3config",
  },
})
```
