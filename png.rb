require 'chunky_png'
require 'pry'
X_DIM = 16
Y_DIM = 16

`mkdir -p sample/solid_colors`
colors = []
(0...8).map {|h| h * 32}.each do |h|
  [0.20, 0.4, 0.6, 0.8, 1.0].each do |s|
    [0.20, 0.4, 0.6, 0.8, 0.9].each do |l|
      colors << [h, s, l]
    end
  end
end
colors << [0,0,0]
colors << [0,1,1]
colors.each do |h, s, l|
  color = ChunkyPNG::Color.from_hsl(h, s, l)
  png = ChunkyPNG::Image.new(X_DIM, Y_DIM, color)
  filename = "sample/solid_colors/#{h}_#{s}_#{l}.png"
  png.save(filename)
  puts "wrote #{filename}"
end
