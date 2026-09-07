import 'dart:convert';

enum StockStatus { available, low, unavailable }

class Product {
  const Product({
    required this.sku,
    required this.name,
    required this.quantity,
    required this.reorderLevel,
  });

  final String sku;
  final String name;
  final int quantity;
  final int reorderLevel;

  factory Product.fromJson(Map<String, Object?> json) {
    return Product(
      sku: json['sku']! as String,
      name: json['name']! as String,
      quantity: json['quantity']! as int,
      reorderLevel: json['reorderLevel']! as int,
    );
  }

  StockStatus get status {
    if (quantity == 0) return StockStatus.unavailable;
    if (quantity <= reorderLevel) return StockStatus.low;
    return StockStatus.available;
  }
}

extension StockStatusLabel on StockStatus {
  String get label => switch (this) {
        StockStatus.available => 'available',
        StockStatus.low => 'low stock',
        StockStatus.unavailable => 'unavailable',
      };
}

Iterable<Product> decodeProducts(String source) sync* {
  final records = jsonDecode(source) as List<Object?>;
  for (final record in records) {
    yield Product.fromJson(record! as Map<String, Object?>);
  }
}

Map<StockStatus, List<Product>> groupByStatus(Iterable<Product> products) {
  final grouped = <StockStatus, List<Product>>{};
  for (final product in products) {
    grouped.putIfAbsent(product.status, () => <Product>[]).add(product);
  }
  return grouped;
}

void printReport(Map<StockStatus, List<Product>> grouped) {
  for (final status in StockStatus.values) {
    final products = grouped[status] ?? const <Product>[];
    print('${status.label}: ${products.length}');
    for (final product in products) {
      print('  ${product.sku} ${product.name} (${product.quantity})');
    }
  }
}

Future<void> main() async {
  const source = '''
  [
    {"sku":"TEA-01","name":"Breakfast tea","quantity":14,"reorderLevel":5},
    {"sku":"MUG-02","name":"Ceramic mug","quantity":3,"reorderLevel":4},
    {"sku":"TIN-03","name":"Tea tin","quantity":0,"reorderLevel":2}
  ]
  ''';

  final products = decodeProducts(source).toList(growable: false);
  printReport(groupByStatus(products));
}
