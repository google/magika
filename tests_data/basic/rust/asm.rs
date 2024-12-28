use std::arch::asm;

fn main() {
 
    let mut x: u64 = 5;

    println!("Original value of x: {}", x);

    unsafe {
        asm!(
            "mov rax, {x}",      
            "mul rax",            
            "mov {x}, rax",       
            x = inout(reg) x,    
        );
    }

    println!("Squared value of x: {}", x);

    assert_eq!(x, 5 * 5);
}
