require 'chunky_png'
X_DIM = 8
Y_DIM = 8
ARGV.map(&:upcase).each do |color|
  filename = "images/#{color}.png"
  png = ChunkyPNG::Image.new(X_DIM, Y_DIM, color)
  png.save(filename)
  puts "wrote #{filename}"
end
