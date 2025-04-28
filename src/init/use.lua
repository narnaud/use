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
local function use_setenv(param)
    local f = io.popen([[::USE::]] .. " " .. param)
    local f = io.popen(command)
    local result = {}
    if f then
        for line in f:lines() do
            -- Set the environment variable
            if line:sub(1, 5) == "SET: " then
                line = line:sub(6)
                local env, value = line:match("([^=]+)=(.*)")
                os.setenv(env, value)
            -- Prepend to the PATH
            elseif line:sub(1, 6) == "PATH: " then
                local path = line:sub(7)
                os.setenv("PATH", path .. ";" .. os.getenv("PATH"))
            -- Change the current directory
            elseif line:sub(1, 4) == "GO: " then
                local dir = line:sub(5)
                table.insert(result, "pushd \"" .. dir .. "\" > nul & echo \x1b[2A")
            -- Execute a command
            elseif line:sub(1, 7) == "DEFER: " then
                local script = line:sub(8)
                table.insert(result, "call \"" .. script .. "\" > nul & echo \x1b[2A")
            elseif line:sub(1, 7) == "TITLE: " then
                local title = line:sub(8)
                console.settitle(title)
            else
                print(line)
            end
        end
        f:close()
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
    if param == "" or param:match("^[-|init|list|set|help]") then
        os.execute([[::USE::]] .. " " .. param)
        return "", false
    else
        return use_setenv(param), false
    end
end
clink.onfilterinput(use_filter)

--------------------------------------------------------------------------------
-- Use Completions, lists all known envs, not commands (init, list...)
local function list_envs()
    local envs
    local r = io.popen([[::USE::]] .. " list")
    envs = {}
    for line in r:lines() do
        table.insert(envs, line)
    end
    return envs
end

clink.argmatcher(table.unpack(string.explode(use_commands or "use")))
:addarg(list_envs())
:addflags("--help", "-h", "--version", "-V")
:nofiles()

--------------------------------------------------------------------------------
-- Set current shell
os.setenv('USE_SHELL', 'cmd')
