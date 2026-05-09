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

-- ==========================================
-- SNAP ASSIST: Window Positioning
-- ==========================================

local function snap_window(position)
    -- Get total screen dimensions
    local sw = mp.get_property_number("display-width")
    local sh = mp.get_property_number("display-height")
    
    if sw == nil or sh == nil then
        mp.msg.error("Could not determine display dimensions.")
        return
    end

    -- Calculate the exact half-way points
    local half_w = math.floor(sw / 2)
    local half_h = math.floor(sh / 2)

    local geo = ""
    local msg = ""

    -- Format for geometry is: Width x Height + X_Position + Y_Position
    if position == "top-left" then
        geo = half_w .. "x" .. half_h .. "+0+0"
        msg = "Snap: Top Left"
    elseif position == "top-right" then
        geo = half_w .. "x" .. half_h .. "+" .. half_w .. "+0"
        msg = "Snap: Top Right"
    elseif position == "bottom-left" then
        geo = half_w .. "x" .. half_h .. "+0+" .. half_h
        msg = "Snap: Bottom Left"
    elseif position == "bottom-right" then
        geo = half_w .. "x" .. half_h .. "+" .. half_w .. "+" .. half_h
        msg = "Snap: Bottom Right"
    elseif position == "left-half" then
        geo = half_w .. "x" .. sh .. "+0+0"
        msg = "Snap: Left Half"
    elseif position == "right-half" then
        geo = half_w .. "x" .. sh .. "+" .. half_w .. "+0"
        msg = "Snap: Right Half"
    end

    -- Apply the geometry and flash the message
    mp.set_property("geometry", geo)
    mp.osd_message(msg, 1)
end

-- Bind Ctrl + Shift + Numbers to the Snap positions
-- Bind Alt + Numbers to the Snap positions
mp.add_forced_key_binding("Alt+1", "snap_tl", function() snap_window("top-left") end)
mp.add_forced_key_binding("Alt+2", "snap_tr", function() snap_window("top-right") end)
mp.add_forced_key_binding("Alt+3", "snap_bl", function() snap_window("bottom-left") end)
mp.add_forced_key_binding("Alt+4", "snap_br", function() snap_window("bottom-right") end)
mp.add_forced_key_binding("Alt+5", "snap_lh", function() snap_window("left-half") end)
mp.add_forced_key_binding("Alt+6", "snap_rh", function() snap_window("right-half") end)