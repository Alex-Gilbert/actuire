local dap = require("dap")

local function get_debugger()
    local debugger = vim.fn.stdpath("data") .. '/mason/bin/codelldb'
    print(debugger)
    return debugger
end

local function get_test_binary_path()
    local command = "cargo test --no-run --message-format=json | jq -r 'select(.profile.test == true) | .filenames[]'"
    local handle = io.popen(command)
    local result = handle:read("*a")
    handle:close()

    -- Assuming the first line contains the path you need
    local test_binary_path = vim.fn.split(result, "\n")[1]
    return vim.fn.trim(test_binary_path)
end

dap.adapters.codelldb = {
    id = 'codelldb',
    type = 'server',
    port = "${port}",
    executable = {
        -- Change this to your path!
        command = get_debugger(),
        args = { "--port", "${port}" },
    }
}

dap.configurations.rust = {
    {
        name = "Debug Actuire",
        type = "codelldb",
        request = "launch",
        program = function()
            -- Run cargo build and capture the output
            local build_output = vim.fn.system("cargo build")

            -- Check if the build was successful
            if vim.v.shell_error ~= 0 then
                print("Build failed:\n" .. build_output)
                return nil -- Returning nil to indicate the build failure
            else
                print("Build successful")
                return vim.fn.getcwd() .. "/target/debug/actuire"
            end
        end,
        cwd = "${workspaceFolder}",
        stopOnEntry = false,
    },
    {
        name = "Debug Rust Tests",
        type = "codelldb",
        request = "launch",
        program = function()
            return get_test_binary_path()
        end,

        cwd = "${workspaceFolder}",
        stopOnEntry = false,
    },
}
