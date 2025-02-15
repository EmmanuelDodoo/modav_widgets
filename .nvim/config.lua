Sessions()

vim.g.builder = "cargo run"

vim.keymap.set("n", "<leader>c", function()
    Run("cargo check")
end)
