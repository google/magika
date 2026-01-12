class Kalimat
  def initialize(nama = "Dunia")
    @nama = nama
  end
  def sapaan
    puts "Hai #{@nama}."
  end
  def perpisahan
    puts "Sampai jumpa #{@nama}."
  end
end
