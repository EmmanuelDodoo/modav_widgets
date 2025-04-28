Sessions()

vim.g.builder = "cargo run --example highlighter"

vim.keymap.set("n", "<leader>c", function()
    Run("cargo clippy")
end)
