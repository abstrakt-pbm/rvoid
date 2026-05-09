````md
# Startup

`startup` — это слой платформенно-зависимого запуска `rvoid`.

Его задача — принять управление от конкретной среды запуска, выполнить необходимую раннюю инициализацию, собрать сведения о системе и привести их к единому виду `rvoid_core::SystemInfo`.

Примеры startup backend-ов:

- UEFI;
- raw x86_64;
- Multiboot2;
- Limine.

## Контракт

Каждый startup backend должен в итоге передать управление пользовательской функции вида:

```rust
fn main(system: rvoid_core::SystemInfo) -> !
````

Все платформенные различия должны быть обработаны до вызова пользовательского кода.

## Обязанности startup backend-а

Startup backend отвечает за:

1. создание внешней точки входа под конкретную платформу;
2. соблюдение ABI и calling convention этой платформы;
3. перевод машины в целевое состояние, если это требуется;
4. получение системной информации из платформенного источника;
5. нормализацию этой информации в `rvoid_core::SystemInfo`;
6. вызов пользовательской функции входа.

## Структура backend-а

Каждый startup backend должен состоять из двух частей:

```text
backend/
  Runtime-код запуска и формирования SystemInfo.

entry_macro/
  Procedural macro для генерации внешней точки входа.
```

`entry_macro` компилируется и выполняется на host-системе во время сборки.

`backend` компилируется под целевую платформу и попадает в итоговый образ.

## Entry macro

Каждый startup backend должен экспортировать attribute macro `entry`.

Пользовательский код должен выглядеть одинаково для всех backend-ов:

```rust
use rvoid::prelude::*;

#[rvoid::entry]
fn main(system: SystemInfo) -> ! {
    loop {}
}
```

Crate `rvoid` переэкспортирует `entry` выбранного backend-а под единым именем:

```rust
#[cfg(feature = "uefi")]
pub use uefi_rvoid::entry;

#[cfg(feature = "raw-x86_64")]
pub use raw_x86_64_rvoid::entry;
```

## Runtime backend

Runtime backend содержит реальную startup-логику:

* получение аргументов от среды запуска;
* ранняя инициализация машины;
* получение карты памяти;
* получение framebuffer, если он доступен;
* поиск firmware tables, если они доступны;
* формирование `rvoid_core::SystemInfo`.

`entry_macro` не должен выполнять эту работу. Он только генерирует внешнюю точку входа и вызывает startup-функцию backend-а.

## Пример UEFI

UEFI backend создаёт точку входа вида:

```rust
extern "efiapi" fn efi_main(
    image_handle: EfiHandle,
    system_table: *mut EfiSystemTable,
) -> EfiStatus
```

Концептуально `#[rvoid::entry]` раскрывается в:

```rust
#[unsafe(no_mangle)]
pub extern "efiapi" fn efi_main(
    image_handle: EfiHandle,
    system_table: *mut EfiSystemTable,
) -> EfiStatus {
    let system = unsafe {
        startup(image_handle, system_table)
    };

    main(system)
}
```

## Правило SystemInfo

`SystemInfo` должен быть полностью backend-independent.

Сырые UEFI-структуры, Multiboot2 tags, Device Tree данные и другие platform-specific структуры не должны попадать в публичный API `SystemInfo`.

Backend может использовать их внутри своей реализации, но наружу должен отдавать только нормализованную модель.

## Инвариант

Любой startup backend обязан завершаться единым контрактом:

```rust
fn main(system: rvoid_core::SystemInfo) -> !
```

После этой точки пользовательский код не должен зависеть от того, через какую среду был запущен `rvoid`.

