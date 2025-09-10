--------------------------------------------------------------------------------
-- Customization.
--
-- use_commands
--      You can set the `use_commands` global variable to a list of command names.
--      The `use` behavior will be assigned to each of the command names.
--
--      For example, you might want both `use` and `use_with` to work:
--          use_commands = "use use_with"

-- luacheck: globals use_commands
use_commands = use_commands or "use"

--------------------------------------------------------------------------------
-- Check if the `use` command is used and extract arguments.
local function use_getparam(line)
    -- Check for "use" command.
    local candidate = line:match("^[ \t]*([^ \t]+)")
    if not candidate then
        return
    end
    local commands = string.explode(use_commands or "use")
    local command_name
    for _,name in ipairs(commands) do
        if candidate == name then
            command_name = name
        end
    end
    if not command_name then
        return
    end

    -- Check for parameters
    local param = line:match("^[ \t]*[^ \t]+[ \t]+(.*)")
    if not param then
        return ""
    end
    return param
end

--------------------------------------------------------------------------------
-- Use execution.
local function get_tmp_script()
    local tmpdir = os.getenv("TMP") or os.getenv("TEMP") or "/tmp"
    local filename = tmpdir .. "/use_clink.bat"
    return filename
end

--------------------------------------------------------------------------------
-- Use execution.
local function use_run(param)
    local use_output = io.popen(::USE:: .. " " .. param)
    local result = {}
    if use_output then
        local filename = get_tmp_script()
        local file = io.open(filename, "w")
        if file then
            file:write("@echo off\n")
            for line in use_output:lines() do
                file:write(line)
                file:write("\n")
            end
            file:write("echo.\n")
            file:close()
            table.insert(result, "call " .. filename .. "& echo \x1b[2A")
        else
            table.insert(result, "echo Failed to create file: " .. filename)
        end
        use_output:close()
    end
    return result
end

--------------------------------------------------------------------------------
-- Filter out input to run the use executable if needed
local function use_filter(line)
    local param = use_getparam(line)
    if not param then
        return
    end

    -- Check for any flags
    if param == "" or param:match("^[-|init|config|list|set|print|help]") then
        os.execute(::USE:: .. " " .. param)
        return "", false
    else
        return use_run(param), false
    end
end
clink.onfilterinput(use_filter)

--------------------------------------------------------------------------------
-- Use Completions, lists all known envs, not commands (init, list...)
local function list_envs()
    local envs
    local r = io.popen(::USE:: .. " list")
    envs = {}
    for line in r:lines() do
        table.insert(envs, line)
    end
    return envs
end

--------------------------------------------------------------------------------
-- Set current shell
os.setenv('USE_SHELL', 'cmd')

--------------------------------------------------------------------------------
-- Set current shell
clink.argmatcher(table.unpack(string.explode(use_commands or "use")))
:addarg(list_envs())
:addflags("--help", "-h", "--version", "-V", "--dependencies", "-d", "--create")
:nofiles()
