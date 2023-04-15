#![no_std]
#![no_main]

use panic_halt as _;
use riscv as _;
use riscv_rt::entry;
use ch32v3::ch32v30x;

const RCC_HPRE_DIV1:u8  = 0b0000;
const RCC_PPRE2_DIV1:u8 = 0b0000;
const RCC_PPRE1_DIV2:u8 = 0b0100;

fn system_init() -> ch32v30x::Peripherals {
    let peripherals = ch32v30x::Peripherals::take().unwrap();
    let rcc = &peripherals.RCC;
    let ext = &peripherals.EXTEND;

    // EXTEN->EXTEN_CTR |= EXTEN_PLL_HSI_PRE;
    ext.extend_ctr.modify( |_,w| w.pll_hsi_pre().set_bit() );

    unsafe {
        // HCLK = SYSCLK
        // RCC->CFGR0 |= (uint32_t)RCC_HPRE_DIV1;
        // rcc.cfgr0.modify(|_,w| w.hpre().bits(RCC_HPRE_DIV1));

        // PCLK2 = HCLK
        // RCC->CFGR0 |= (uint32_t)RCC_PPRE2_DIV1;
        // rcc.cfgr0.modify(|_,w| w.ppre2().bits(RCC_PPRE2_DIV1));

        // PCLK1 = HCLK
        // RCC->CFGR0 |= (uint32_t)RCC_PPRE1_DIV2;
        // rcc.cfgr0.modify(|_,w| w.ppre1().bits(RCC_PPRE1_DIV2));

        //  PLL configuration: PLLCLK = HSI * 9 = 72 MHz
        // RCC->CFGR0 &= (uint32_t)((uint32_t)~(RCC_PLLSRC | RCC_PLLXTPRE | RCC_PLLMULL));
        // rcc.cfgr0.modify(|_,w| w.pllsrc().set_bit().pllxtpre().set_bit());
        // RCC->CFGR0 |= (uint32_t)(RCC_PLLSRC_HSI_Div2 | RCC_PLLMULL9);
        rcc.cfgr0.modify(|_, w| w.pllmul().bits(0b0111));
    }

    // Enable PLL
    // RCC->CTLR |= RCC_PLLON;
    rcc.ctlr.modify( |_,w| w.pllon().set_bit());
    // Wait till PLL is ready
    while rcc.ctlr.read().pllrdy() != true {}
    // Select PLL as system clock source
    rcc.cfgr0.modify(|_, w| unsafe  { w.sw().bits(0b10) });
    // Wait till PLL is used as system clock source
    while rcc.cfgr0.read().sws() != 0x02 {}

    peripherals
}

fn init_led(phps:&ch32v30x::Peripherals) {
    let rcc = &phps.RCC;
    rcc.apb2pcenr.modify(|_,w| w.iopaen().set_bit())
}

#[entry]
fn main() -> ! {

    let peripherals = system_init();

    init_led(&peripherals);

    let gpioa = &peripherals.GPIOA;

    gpioa.cfghr.modify(|_,w| unsafe {
        w.cnf15().bits(0b00)  // GPIO_Speed_50MHz
         .mode15().bits(0b11) // GPIO_Mode_Out_PP
    });

    unsafe {
        riscv::interrupt::enable();
    }

    let cycle = 72_000_000 / 2;
    // GPIO_ResetBits
    gpioa.bcr.write( |w| w.br15().set_bit());
    // gpioa.bshr.write( |w| w.bs15().set_bit());

    loop {
        gpioa.bshr.write( |w| w.bs15().set_bit());
        unsafe { riscv::asm::delay(cycle) }
        gpioa.bcr.write( |w| w.br15().set_bit());
        unsafe { riscv::asm::delay(cycle) }
    }
}
