Sessions()

vim.g.builder = "cargo run --example tree"

vim.keymap.set("n", "<leader>c", function()
    Run("cargo clippy")
end)
