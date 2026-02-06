
const FREEZING: f64 = 32.0;

fn farenheit_to_celsius(f:f64) ->f64{
    (f- FREEZING) * (5.0/9.0)
}

fn celsius_to_farenheit(c:f64) ->f64{
    (c*(9.0/5.0)) + 32.0
}


fn ass1(){
    let temp:f64 = 98.0;
    println!("{}",farenheit_to_celsius(temp));

    let mut x = temp as i64 + 1;
    for n in x..x+5{
        println!("{}",farenheit_to_celsius(n as f64));
    }
    println!("\nEND OF ASSIGNEMENT 1");
    println!("---------------------------------");
}

//---------------------------------------------------------------

fn is_even(n:i32) -> bool{
    n%2==0
}

fn ass2(){
    let arr: [i32; 10] = [10,15,25,40,65,105,180,285,465,750];
    for num in arr{
        println!("{}: ",num);
        if(is_even(num)){
            println!("Even");
        }
        else{
            println!("Odd");
        }
        if(num%5==0 && num%3==0){
            println!("FizzBuzz\n");
            continue;
        }
        if(num%3==0){
            println!("Fizz\n");
        }
        if(num%5==0){
            println!("Buzz\n");
        }
    }

    let mut sum = 0;
    let mut x = 0;

    while (x<arr.len()){
        sum+=arr[x];
        x+=1;
    }
    println!("Sum of array: {}", sum);

    let mut big = arr[0];
    for num in arr{
        if(big < num){
            big = num;
        }        
    }

    println!("Largest Number: {}\n", big);
    println!("END OF ASSIGNMENT 2");
    println!("---------------------------------");

}




//---------------------------------------------------------------


fn check_guess(guess: i32, secret: i32) -> i32{
    if(guess == secret){
        return 0
    }
    if(guess > secret){
        return 1
    }
    else{
        return -1
    }
}

fn ass3(){
    let mut secret = 13;
    let mut guess = 20;
    let mut status = check_guess(guess, secret);
    let mut tries = 1;
    loop{
        if(status == 0){
            break;
        }
        if(status == 1){
            guess-=2;
        }
        if(status == -1){
            guess+=1;
        }
        status = check_guess(guess, secret);
        tries += 1;
    }
    println!("Attempts needed: {}", tries);
    println!("\nEND OF ASSIGNMENT 3");
    println!("---------------------------------");

}

fn main(){
    ass1();
    ass2();
    ass3();
}