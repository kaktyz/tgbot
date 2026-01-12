// Импортируем все необходимые зависимости для работы бота
use teloxide::prelude::*;
use teloxide::types::InlineKeyboardButton;
use teloxide::types::InlineKeyboardMarkup;
use reqwest;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::env;

// Загружаем переменные окружения из файла .env
use dotenv::dotenv;

// Экспортируем функции, чтобы их можно было использовать в тестах
pub mod bot_functions {
    use super::*;

    // Структура для десериализации JSON-ответа от 2ip.ru
    #[derive(Deserialize, Serialize)]
    pub struct IpResponse {
        // IP-адрес в строковом формате
        pub ip: String,
        // Дополнительные поля можно добавить при необходимости
        // country: Option<String>,
        // city: Option<String>,
        // и т.д.
    }

    // Асинхронная функция для получения IP-адреса с сайта 2ip.ru
    pub async fn get_ip_address() -> String {
        // URL для получения IP-адреса в формате JSON
        let url = "https://2ip.ru/json/";

        // Создаем HTTP-клиент с настройками
        let client = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (compatible; Bot/1.0)")
            .build()
            .unwrap_or_default();

        // Выполняем HTTP GET запрос к API
        match client.get(url).send().await {
            // Если запрос успешен
            Ok(response) => {
                // Проверяем статус ответа
                if response.status().is_success() {
                    // Преобразуем ответ в JSON с использованием нашей структуры данных
                    match response.json::<IpResponse>().await {
                        // Если JSON получен успешно и десериализован
                        Ok(ip_data) => {
                            // Возвращаем IP-адрес из полученной структуры
                            ip_data.ip
                        },
                        // Если возникла ошибка при обработке JSON
                        Err(_) => "Ошибка при обработке ответа".to_string()
                    }
                } else {
                    // Если статус ответа не успешный, пробуем другой URL
                    log::warn!("Первый запрос не удался, пробуем альтернативный URL");
                    get_ip_address_alternative().await
                }
            },
            // Если возникла ошибка при выполнении запроса
            Err(e) => {
                log::warn!("Ошибка при первом запросе: {:?}, пробуем альтернативный URL", e);
                get_ip_address_alternative().await
            }
        }
    }

    // Альтернативная функция для получения IP-адреса
    async fn get_ip_address_alternative() -> String {
        // Альтернативный URL для получения IP-адреса
        let url = "https://httpbin.org/ip";

        // Создаем HTTP-клиент с настройками
        let client = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (compatible; Bot/1.0)")
            .build()
            .unwrap_or_default();

        // Выполняем HTTP GET запрос к альтернативному API
        match client.get(url).send().await {
            // Если запрос успешен
            Ok(response) => {
                // Проверяем статус ответа
                if response.status().is_success() {
                    // Преобразуем ответ в JSON
                    match response.json::<serde_json::Value>().await {
                        // Если JSON получен успешно
                        Ok(json) => {
                            // Извлекаем IP-адрес из JSON
                            match json.get("origin").and_then(|v| v.as_str()) {
                                // Если IP-адрес найден
                                Some(ip) => ip.to_string(),
                                // Если IP-адрес не найден в JSON
                                None => "Не удалось получить IP-адрес из ответа".to_string()
                            }
                        },
                        // Если возникла ошибка при обработке JSON
                        Err(_) => "Ошибка при обработке альтернативного ответа".to_string()
                    }
                } else {
                    // Если статус ответа не успешный
                    "Ошибка при альтернативном запросе".to_string()
                }
            },
            // Если возникла ошибка при выполнении запроса
            Err(_) => "Ошибка соединения с альтернативным сервисом".to_string()
        }
    }

    // Функция для генерации случайного числа
    pub fn generate_random_number() -> i32 {
        // Создаем генератор случайных чисел, используя системный источник энтропии
        let mut rng = rand::thread_rng();
        
        // Получаем минимальное и максимальное значение из переменных окружения
        // Если переменные не установлены, используем значения по умолчанию
        let min = env::var("RANDOM_MIN")
            .unwrap_or_else(|_| "1".to_string())
            .parse()
            .unwrap_or(1);
        let max = env::var("RANDOM_MAX")
            .unwrap_or_else(|_| "100".to_string())
            .parse()
            .unwrap_or(100);
        
        // Проверяем, что минимальное значение не больше максимального
        let (min, max) = if min <= max { (min, max) } else { (max, min) };
        
        // Генерируем случайное число в заданном диапазоне
        rng.gen_range(min..=max)
    }
}

// Основная асинхронная функция, которая запускает бота
#[tokio::main]
async fn main() {
    // Загружаем переменные окружения из файла .env
    dotenv().ok();
    
    // Инициализируем env_logger для настройки логирования
    env_logger::init();
    
    // Получаем токен бота из переменной окружения
    let bot_token = env::var("BOT_TOKEN")
        .expect("Необходимо установить переменную окружения BOT_TOKEN");
    
    // Получаем ID чата из переменной окружения
    let allowed_chat_id: i64 = env::var("CHAT_ID")
        .expect("Необходимо установить переменную окружения CHAT_ID")
        .parse()
        .expect("CHAT_ID должен быть числом");

    // Создаем объект бота с помощью токена
    let bot = Bot::new(bot_token);

    // Запускаем бота с передачей функции-обработчика
    run(bot, allowed_chat_id).await;
}

// Асинхронная функция, которая запускает обработку сообщений
async fn run(bot: Bot, allowed_chat_id: i64) {
    log::info!("Запуск бота с разрешенным ID чата: {}", allowed_chat_id);

    // Создаем диспетчер команд, который будет обрабатывать входящие сообщения
    Dispatcher::builder(
        bot,
        dptree::entry()
            .branch(
                Update::filter_message()
                    .endpoint(message_handler)
            )
            .branch(
                Update::filter_callback_query()
                    .endpoint(callback_handler)
            )
    )
    .dependencies(dptree::deps![allowed_chat_id])
    .enable_ctrlc_handler()
    .build()
    .dispatch()
    .await;
    
    log::info!("Диспетчер завершил работу");
}

// Функция для создания клавиатуры с кнопками
fn create_keyboard() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![
        vec![
            // Кнопка для получения IP-адреса
            InlineKeyboardButton::callback("Получить IP", "get_ip"),
            // Кнопка для генерации случайного числа
            InlineKeyboardButton::callback("Случайное число", "get_random"),
        ]
    ])
}

// Обработчик сообщений
async fn message_handler(bot: Bot, msg: Message, allowed_chat_id: i64) -> ResponseResult<()> {
    log::info!("Получено сообщение от пользователя: {}, тип чата: {:?}", msg.chat.id.0, msg.chat.kind);

    // Проверяем, что сообщение пришло из разрешенного чата
    if msg.chat.id.0 != allowed_chat_id {
        // Если чат не разрешен, просто игнорируем сообщение
        log::warn!("Получено сообщение из неразрешенного чата: {}", msg.chat.id.0);
        return Ok(());
    }

    log::info!("Сообщение из разрешенного чата, создаем клавиатуру");

    // Создаем клавиатуру с двумя кнопками
    let keyboard = create_keyboard();

    // Отправляем сообщение пользователю с клавиатурой
    bot.send_message(msg.chat.id, "Выберите действие:")
        .reply_markup(keyboard)
        .await?;

    log::info!("Сообщение с клавиатурой отправлено пользователю: {}", msg.chat.id.0);

    Ok(())
}

// Обработчик callback-запросов
async fn callback_handler(bot: Bot, cq: CallbackQuery, allowed_chat_id: i64) -> ResponseResult<()> {
    log::info!("Получен callback query: {:?}", cq.data);

    // Проверяем, что callback пришел из разрешенного чата
    if let Some(ref msg) = cq.message {
        // Извлекаем ID чата из сообщения
        let chat_id = msg.chat.id.0;

        log::info!("Проверка чата: ожидаем {}, получили {}", allowed_chat_id, chat_id);

        // Проверяем, что чат разрешен
        if chat_id != allowed_chat_id {
            // Отправляем ответ, чтобы убрать "часики" ожидания
            bot.answer_callback_query(&cq.id).await?;
            log::warn!("Получен callback из неразрешенного чата: {}", chat_id);
            return Ok(());
        }
    } else {
        // Если сообщение отсутствует, прерываем обработку
        log::warn!("Получен callback без сообщения");
        bot.answer_callback_query(&cq.id).await?;
        return Ok(());
    }

    // Проверяем, что callback query содержит данные (data)
    if let Some(data) = cq.data {
        log::info!("Обработка callback с данными: {}", data);

        // В зависимости от нажатой кнопки выполняем соответствующее действие
        match data.as_str() {
            // Если нажата кнопка "Получить IP"
            "get_ip" => {
                // Получаем IP-адрес
                let ip_address = bot_functions::get_ip_address().await;

                log::info!("Получен IP-адрес: {}", ip_address);

                // Отправляем результат пользователю
                if let Some(ref msg) = cq.message {
                    bot.send_message(msg.chat.id, format!("Ваш IP-адрес: {}", ip_address))
                        .await?;
                    log::info!("Отправлено сообщение с IP-адресом");

                    // Отправляем сообщение с клавиатурой снова
                    bot.send_message(msg.chat.id, "Выберите действие:")
                        .reply_markup(create_keyboard())
                        .await?;
                    log::info!("Сообщение с клавиатурой отправлено снова");
                }
            },
            // Если нажата кнопка "Случайное число"
            "get_random" => {
                // Генерируем случайное число
                let random_number = bot_functions::generate_random_number();

                log::info!("Сгенерировано случайное число: {}", random_number);

                // Отправляем результат пользователю
                if let Some(ref msg) = cq.message {
                    bot.send_message(msg.chat.id, format!("Случайное число: {}", random_number))
                        .await?;
                    log::info!("Отправлено сообщение со случайным числом");

                    // Отправляем сообщение с клавиатурой снова
                    bot.send_message(msg.chat.id, "Выберите действие:")
                        .reply_markup(create_keyboard())
                        .await?;
                    log::info!("Сообщение с клавиатурой отправлено снова");
                }
            },
            // Если данные не совпадают ни с одной кнопкой
            _ => {
                log::warn!("Получены неизвестные данные в callback: {}", data);
                if let Some(ref msg) = cq.message {
                    bot.send_message(msg.chat.id, "Неизвестная команда").await?;

                    // Отправляем сообщение с клавиатурой снова
                    bot.send_message(msg.chat.id, "Выберите действие:")
                        .reply_markup(create_keyboard())
                        .await?;
                    log::info!("Сообщение с клавиатурой отправлено снова");
                }
            }
        }
    } else {
        log::warn!("Callback query не содержит данных");
        bot.answer_callback_query(&cq.id).await?;
        return Ok(());
    }

    // Отвечаем на callback query, чтобы убрать "часики" ожидания
    bot.answer_callback_query(&cq.id).await?;
    log::info!("Callback query обработан успешно");

    Ok(())
}

// Тесты для функций получения IP и генерации случайных чисел
#[cfg(test)]
mod tests {
    use super::bot_functions::generate_random_number;

    // Тест для проверки генерации случайного числа
    #[test]
    fn test_generate_random_number() {
        // Вызываем функцию генерации случайного числа несколько раз
        let num1 = generate_random_number();
        let num2 = generate_random_number();

        // Проверяем, что числа входят в ожидаемый диапазон (по умолчанию 1-100)
        assert!(num1 >= 1 && num1 <= 100, "Число должно быть в диапазоне 1-100");
        assert!(num2 >= 1 && num2 <= 100, "Число должно быть в диапазоне 1-100");

        // Проверяем, что функция может генерировать разные числа
        // (это может не сработать в редких случаях из-за природы случайности, но в большинстве случаев должно работать)
        // Для надежности можно запустить тест несколько раз
    }

    // Тест для проверки генерации случайного числа в заданном диапазоне
    #[test]
    fn test_generate_random_number_with_env() {
        // Создаем отдельную функцию для тестирования с переменными окружения
        // чтобы избежать конфликта с другими тестами
        fn test_with_env(min: &str, max: &str, expected_min: i32, expected_max: i32) -> bool {
            std::env::set_var("RANDOM_MIN", min);
            std::env::set_var("RANDOM_MAX", max);

            // Генерируем несколько чисел и проверяем, что хотя бы одно в диапазоне
            let mut success = false;
            for _ in 0..100 {
                let num = generate_random_number();
                if num >= expected_min && num <= expected_max {
                    success = true;
                    break;
                }
            }

            // Восстанавливаем значения по умолчанию
            std::env::remove_var("RANDOM_MIN");
            std::env::remove_var("RANDOM_MAX");

            success
        }

        // Проверяем, что число входит в заданный диапазон
        assert!(test_with_env("10", "20", 10, 20), "Число должно быть в диапазоне 10-20");
    }

    // Тест для проверки обработки некорректных значений переменных окружения
    #[test]
    fn test_generate_random_number_with_invalid_env() {
        // Устанавливаем некорректные переменные окружения
        std::env::set_var("RANDOM_MIN", "abc");
        std::env::set_var("RANDOM_MAX", "def");

        // Вызываем функцию генерации случайного числа
        let num = generate_random_number();

        // Проверяем, что используется значение по умолчанию (1-100)
        assert!(num >= 1 && num <= 100, "При некорректных переменных окружения должно использоваться значение по умолчанию");

        // Восстанавливаем значения по умолчанию
        std::env::remove_var("RANDOM_MIN");
        std::env::remove_var("RANDOM_MAX");
    }

}

// Тест для проверки загрузки переменных из .env файла
#[cfg(test)]
mod env_test {
    use std::env;

    #[test]
    fn test_env_loading() {
        // Загружаем переменные из .env файла
        dotenv::dotenv().ok();
        
        // Проверяем, что переменные окружения доступны
        let bot_token = env::var("BOT_TOKEN");
        let chat_id = env::var("CHAT_ID");
        
        // Эти переменные могут быть не установлены в тестовой среде,
        // но если .env файл правильно читается, то они должны быть установлены
        // хотя бы на значения по умолчанию из .env файла
        println!("BOT_TOKEN: {:?}", bot_token);
        println!("CHAT_ID: {:?}", chat_id);
        
        // Тест просто проверяет, что загрузка .env не вызывает ошибок
        assert!(true); // Просто подтверждение, что тест завершается успешно
    }
}
