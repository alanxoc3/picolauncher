pico-8 cartridge // http://www.pico-8.com
version 42
__lua__

#include menu.p8
#include utils.p8

-- screenshot and recording viewer

photo_dir='screenshots' 

-- constants
local grid_cols = 3
local grid_rows = 4
local thumb_size = 32
local grid_offset_x = 10
local grid_offset_y = 20

-- state
local photos = {}
local current_photo = 1
local is_fullscreen = false
local scroll_y = 0

function _init()
  cartdata("screenshot_gallery_data")
  load_photos()
end

function _update()
  if is_fullscreen then
    update_fullscreen()
  else
    update_gallery()
  end
end

function _draw()
  cls(1)  -- dark blue background
  if is_fullscreen then
    draw_fullscreen()
  else
    draw_gallery()
  end
end

function update_gallery()
  if btnp(2) then  
    current_photo = max(1, current_photo - grid_cols)
  elseif btnp(3) then  -- down
    current_photo = min(tsize(photos), current_photo + grid_cols)
  elseif btnp(0) then  -- left
    current_photo = max(1, current_photo - 1)
  elseif btnp(1) then  -- right
    current_photo = min(tsize(photos), current_photo + 1)
  elseif btnp(5) then  -- x button
    is_fullscreen = true
  end
   
  local target_y = flr((current_photo - 1) / grid_cols) * (thumb_size + 4) - 32
  scroll_y = scroll_y + (target_y - scroll_y) / 4
end

-- draw thumbnail n at (x,y)
function draw_thumbnail(n, x, y)
  if n >= 0 then
    for j = 0, 7 do
      for i = 0, 3 do
        sspr(i*32, n*8 + j, 32, 1, x, y+j*4+i)
      end
    end
  end
end

function draw_gallery()
  -- draw header
  rectfill(0, 0, 128, 8, 9) 
  print("photos", 2, 2, 7)
  -- print("❎ screenshot", 75, 5, 7)
   
  for i, photo in ipairs(loaded_thumbnails) do
    local col = (i-1) % grid_cols
    local row = flr((i-1) / grid_cols)
    local x = grid_offset_x + col * (thumb_size + 4)
    local y = grid_offset_y + row * (thumb_size + 4) - scroll_y
    
    if y > 16 and y < 128 then
      rectfill(x, y, x+thumb_size-1, y+thumb_size-1, 0)
      draw_thumbnail(i-1, x, y)
      print(i, x+2, y+2, 7)
      
      -- highlight selected photo
      if i == current_photo then
        rect(x-1, y-1, x+thumb_size, y+thumb_size, 7)
      end
    end
  end
end

function update_fullscreen()
  if btnp(5) then  
    is_fullscreen = false
  elseif btnp(0) then  
    current_photo = max(1, current_photo - 1)
  elseif btnp(1) then  
    current_photo = min(tsize(photos), current_photo + 1)
  end
end

function draw_fullscreen()
  local photo = photos[current_photo]
  rectfill(0, 0, 127, 127, 13)
  print("screenshot "..photo.index, 32, 60, 7)
  
  rectfill(0, 120, 127, 127, 9)  -- orange bar
  print("photo_"..photo.index..".png", 4, 122, 7)
  print("⬅️➡️", 110, 122, 7)
end

loaded_thumbnails={}
function load_photos()
  photos={} -- reset photos
  loaded_thumbnails={}
  for i, path in ipairs(ls(photo_dir)) do
    if ends_with(path, ".128.p8") then
      stripped_path = sub(path, 0, -#(".128.p8")-1)
      photos[stripped_path] = true
    end
    if ends_with(path, ".32.p8") then
      stripped_path = sub(path, 0, -#(".32.p8")-1)
      photos[stripped_path] = true
      if #loaded_thumbnails < 9 then
        reload(#loaded_thumbnails*0x0200, 0x0000, 0x0200, photo_dir .. "/" .. path)
        printh(photo_dir .. "/" .. path)
        add(loaded_thumbnails, path)
      end
    end
  end
end

