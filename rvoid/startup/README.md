# Startup

`startup` — это слой платформенно-зависимого запуска `rvoid`.

Его задача — принять управление от конкретной среды запуска, выполнить раннюю инициализацию, собрать сведения о системе, подготовить базовый allocator памяти и привести системную информацию к единому виду `rvoid_core::SystemInfo`.

К средам запуска могут относиться:

- UEFI;
- raw x86_64;
- Multiboot2;
- Limine;
- другие boot protocol-ы.

## Контракт

Любой startup backend должен в итоге передать управление пользовательской функции вида:

```rust
fn main(system: rvoid_core::SystemInfo) -> !
```

Все платформенные различия должны быть обработаны до вызова пользовательского кода.

Пользовательский код не должен зависеть от того, через какую среду был запущен `rvoid`.

## Общая схема

Startup-слой имеет фасадный crate `rvoid-startup`.

Он инкапсулирует конкретные startup backend-ы и выбирает нужный backend по feature.

Концептуальная схема:

```text
rvoid
  -> rvoid-startup
      -> rvoid-uefi
          -> rvoid-uefi-entry
          -> rvoid-uefi-backend
```

Корневой crate `rvoid` не должен напрямую знать внутреннюю структуру конкретного backend-а. Он использует `rvoid-startup` как единый фасад startup-слоя.

## Обязанности startup backend-а

Startup backend отвечает за:

1. создание внешней точки входа под конкретную платформу;
2. соблюдение ABI и calling convention этой платформы;
3. перевод машины в целевое состояние, если это требуется;
4. получение системной информации из платформенного источника;
5. получение карты памяти или другого источника сведений о доступной памяти;
6. определение начальных занятых диапазонов памяти;
7. выбор и резервирование памяти под служебные структуры базового allocator-а;
8. формирование начального состояния базового allocator-а;
9. нормализацию системной информации в `rvoid_core::SystemInfo`;
10. вызов пользовательской функции входа.

## Структура backend-а

Каждый startup backend должен иметь фасадный crate и две внутренние части:

```text
<backend>/
  Cargo.toml
  src/lib.rs

  backend/
    Cargo.toml
    src/lib.rs

  entry_macro/
    Cargo.toml
    src/lib.rs
```

Фасадный crate backend-а объединяет `entry_macro` и `backend` в единый backend API.

Например для UEFI:

```text
startup/
  uefi/
    Cargo.toml          # rvoid-uefi
    src/lib.rs

    backend/
      Cargo.toml        # rvoid-uefi-backend
      src/lib.rs

    entry_macro/
      Cargo.toml        # rvoid-uefi-entry
      src/lib.rs
```

`entry_macro` компилируется и выполняется на host-системе во время сборки.

`backend` компилируется под целевую платформу и попадает в итоговый bootable image.

## Entry macro

Каждый startup backend должен экспортировать attribute macro `entry`.

Пользовательский код должен выглядеть одинаково для всех backend-ов:

```rust
#![no_std]
#![no_main]

use rvoid::prelude::*;

#[rvoid::entry]
fn main(system: SystemInfo) -> ! {
    loop {}
}
```

`rvoid-startup` переэкспортирует `entry` выбранного backend-а под единым именем:

```rust
#[cfg(feature = "uefi")]
pub use rvoid_uefi::entry as entry;

#[cfg(not(any(feature = "uefi")))]
pub use rvoid_stub_entry::entry as entry;
```

Корневой crate `rvoid` затем переэкспортирует этот macro наружу:

```rust
pub use rvoid_startup::entry;
```

Таким образом пользователь всегда использует `#[rvoid::entry]`, а конкретная форма внешней точки входа определяется выбранным startup backend-ом.

## Runtime backend

Runtime backend содержит реальную startup-логику.

Он отвечает за:

- получение аргументов от среды запуска;
- раннюю инициализацию машины;
- получение карты памяти;
- определение доступных и занятых диапазонов памяти;
- инициализацию базового allocator-а;
- получение framebuffer, если он доступен;
- поиск firmware tables, если они доступны;
- формирование `rvoid_core::SystemInfo`.

`entry_macro` не должен выполнять эту работу.

`entry_macro` только генерирует внешнюю точку входа, соответствующую ABI конкретной платформы, вызывает startup-функцию backend-а и передаёт полученный `SystemInfo` пользовательской функции.

## Инициализация базового allocator-а

`rvoid` предоставляет общий базовый allocator памяти, однако его начальное состояние формируется startup backend-ом.

Startup backend знает источник памяти конкретной платформы и поэтому отвечает за bootstrap allocator-а.

Startup backend должен:

- получить карту памяти или эквивалентный источник сведений о физической памяти;
- определить доступные диапазоны памяти;
- определить занятые диапазоны памяти;
- зарезервировать сам загруженный образ `rvoid` или ядра;
- зарезервировать framebuffer, firmware tables, ACPI, Device Tree и другие специальные области, если они присутствуют;
- выбрать память под служебные структуры allocator-а;
- пометить память allocator metadata как занятую;
- передать allocator-у storage для списков свободных и занятых блоков;
- передать allocator-у начальные available и reserved regions.

После этого базовый allocator считается инициализированным.

Сам allocator не должен самостоятельно выбирать источник памяти. Он получает начальное состояние от startup backend-а и остаётся backend-independent.

## Фасад backend-а

Фасад backend-а объединяет `entry_macro` и `backend`.

Например `rvoid-uefi` экспортирует:

```rust
pub use rvoid_uefi_entry::entry;

pub mod startup {
    pub use rvoid_uefi_backend::{
        startup,
        EfiHandle,
        EfiStatus,
        EfiSystemTable,
    };
}
```

`rvoid-startup` использует этот фасад и не зависит напрямую от внутренних частей UEFI backend-а.

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

В текущей архитектуре сгенерированный UEFI entry-код обращается к startup runtime через путь вида:

```rust
::rvoid::startup::uefi::startup(...)
```

Этот путь предоставляется через фасады:

```text
rvoid
  -> rvoid-startup
      -> rvoid-uefi
          -> rvoid-uefi-backend
```

UEFI backend отвечает за:

- ABI `extern "efiapi"`;
- символ `efi_main`;
- получение `image_handle`;
- получение `system_table`;
- получение UEFI memory map;
- определение диапазона загруженного image;
- резервирование памяти, занятой image;
- подготовку базового allocator-а;
- формирование `rvoid_core::SystemInfo`.

## Stub backend

Если startup backend не выбран, используется stub entry macro.

Его задача — выдать понятную ошибку компиляции:

```text
rvoid: no startup backend selected. Enable one startup backend feature, for example `uefi`.
```

Stub backend не содержит runtime startup-логики и не формирует `SystemInfo`.

## Правило SystemInfo

`SystemInfo` должен быть полностью backend-independent.

Сырые UEFI-структуры, Multiboot2 tags, Device Tree данные и другие platform-specific структуры не должны попадать в публичный API `SystemInfo`.

Backend может использовать их внутри своей реализации, но наружу должен отдавать только нормализованную модель.

## Инвариант

Любой startup backend обязан завершаться единым контрактом:

```rust
fn main(system: rvoid_core::SystemInfo) -> !
```

До этой точки backend может использовать любые необходимые платформенные механизмы.

После этой точки пользовательский код не должен зависеть от того, через какую среду был запущен `rvoid`.
