require 'chunky_png'
X_DIM = 16
Y_DIM = 16
ARGV.map(&:upcase).each do |color|
  filename = "color_images/#{color}.png"
  png = ChunkyPNG::Image.new(X_DIM, Y_DIM, color)
  png.save(filename)
  puts "wrote #{filename}"
end
