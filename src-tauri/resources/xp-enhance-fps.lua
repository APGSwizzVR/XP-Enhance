-- XP Enhance in-simulator FPS overlay
-- Original XP Enhance integration script. Requires FlyWithLua NG+.
local XP_ENHANCE_FPS_ENABLED = __FPS_ENABLED__
local XP_ENHANCE_FPS_POSITION = "__FPS_POSITION__"

dataref("xp_enhance_frame_period", "sim/operation/misc/frame_rate_period", "readonly")

function xp_enhance_draw_fps()
    if not XP_ENHANCE_FPS_ENABLED then return end
    local frame_period = tonumber(xp_enhance_frame_period) or 0
    if frame_period <= 0 then return end
    local fps = math.floor((1.0 / frame_period) + 0.5)
    local label = string.format("XP ENHANCE  %d FPS", fps)
    local screen_w = SCREEN_WIDTH or 1920
    local screen_h = SCREEN_HEIGHT or 1080
    local text_w = #label * 9
    local x = 24
    local y = screen_h - 42
    if XP_ENHANCE_FPS_POSITION == "top-right" then x = screen_w - text_w - 28 end
    if XP_ENHANCE_FPS_POSITION == "bottom-left" then y = 34 end
    if XP_ENHANCE_FPS_POSITION == "bottom-right" then x = screen_w - text_w - 28; y = 34 end
    local colour = "green"
    if fps < 25 then colour = "red" elseif fps < 40 then colour = "yellow" end
    draw_string(x, y, label, colour)
end

do_every_draw("xp_enhance_draw_fps()")
