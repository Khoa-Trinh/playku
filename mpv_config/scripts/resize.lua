local mp = require 'mp'

local function set_window_size(percent)
    -- 1. Get the exact pixel width of the current monitor
    local screen_width = mp.get_property_number("display-width")
    
    if screen_width == nil then
        mp.msg.error("Could not determine display width.")
        return
    end

    -- 2. Calculate the target width based on your percentage
    local width = math.floor(screen_width * (percent / 100))
    
    -- 3. Calculate the height to force a perfect 16:9 aspect ratio
    local height = math.floor(width * (9 / 16))

    -- 4. Apply the new size without moving the window
    mp.set_property("geometry", width .. "x" .. height)
    
    -- 5. Flash a quick on-screen message
    mp.osd_message("Window Size: " .. percent .. "%", 1)
end

-- Bind Ctrl + Numbers to specific screen width percentages
mp.add_forced_key_binding("Ctrl+1", "size_10", function() set_window_size(10) end)
mp.add_forced_key_binding("Ctrl+2", "size_20", function() set_window_size(20) end)
mp.add_forced_key_binding("Ctrl+3", "size_30", function() set_window_size(30) end)
mp.add_forced_key_binding("Ctrl+4", "size_40", function() set_window_size(40) end)
mp.add_forced_key_binding("Ctrl+5", "size_50", function() set_window_size(50) end)
mp.add_forced_key_binding("Ctrl+6", "size_60", function() set_window_size(60) end)
mp.add_forced_key_binding("Ctrl+7", "size_70", function() set_window_size(70) end)
mp.add_forced_key_binding("Ctrl+8", "size_80", function() set_window_size(80) end)
mp.add_forced_key_binding("Ctrl+9", "size_90", function() set_window_size(90) end)
mp.add_forced_key_binding("Ctrl+0", "size_100", function() set_window_size(100) end)