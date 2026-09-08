package com.example.inventory

sealed trait ItemStatus
case object InStock extends ItemStatus
case object OutOfStock extends ItemStatus
case object Discontinued extends ItemStatus

final case class Item(
  id: Long,
  name: String,
  price: BigDecimal,
  quantity: Int,
  status: ItemStatus
)

class InventoryService {
  private var items: Map[Long, Item] = Map.empty

  def addItem(item: Item): Unit = {
    items = items + (item.id -> item)
  }

  def findById(id: Long): Option[Item] = items.get(id)

  def availableItems(): List[Item] = {
    items.values.filter(item => item.quantity > 0 && item.status == InStock).toList
  }

  def totalInventoryValue(): BigDecimal = {
    items.values.foldLeft(BigDecimal(0)) { (acc, item) =>
      acc + (item.price * item.quantity)
    }
  }
}
