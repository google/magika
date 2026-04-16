import java.time.LocalDate;
import java.util.List;

public final class Greeter {
  private Greeter() {}

  public static void main(String[] args) {
    List<String> names = List.of("Ada", "Grace", "Katherine");
    for (String name : names) {
      System.out.printf("Hello %s, today is %s.%n", name, LocalDate.now());
    }
  }
}
