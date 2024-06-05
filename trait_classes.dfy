// Trait object is implicit for everything. Conceptually, object is the Dafny object for
// "std::any::Any"

trait Animal {
  method BecomeOlder() returns (f: () ~> string)
    ensures f.requires() && f.reads() == {this}
    modifies this
}
trait EggLayer extends Animal {
  method Lay1Egg()
  method Lay2Eggs() {
    Lay1Egg();
    Lay1Egg();
  }
}
trait Mammal extends Animal {
  method GiveMilk()
}
trait Platypus extends Animal, EggLayer, Mammal {
}

class NorthernPlatypus extends Platypus {
  var age: string
  method Lay1Egg() {
    print "A northern platypus layed one egg";
  }
  method GiveMilk() {
    print "A norther platypus gave milk";
  }
  method BecomeOlder() returns (f: () ~> string)
    ensures f.requires() && f.reads() == {this}
    modifies this
  {
    this.age := "1" + this.age;
    return () reads this => "I am " + this.age + " years old";
  }
}

method Main() {
  var plato := new NorthernPlatypus;
  plato.age := "4";
  // Upcasting from class to trait
  var p: Platypus := plato;
  // Upcasting from trait to trait
  var eg: EggLayer := p;
  // Upcasting from trait to object
  var o: object := eg;
  // Upcasting from another path
  var a: Animal := plato as Platypus as Mammal;

  // Downcasting from object to trait
  var m2 := o as Mammal; // Not implemented yet
  // Downcasting from trait to trait
  var m3 := a as Mammal;
  // Downcasting from trait to class
  var np := a as NorthernPlatypus;

  // Method calls
  var f := a.BecomeOlder();
  print f();
  eg.Lay2Eggs();
}