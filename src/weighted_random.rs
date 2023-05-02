/**
 * Случайно выбирает один из элементов на основе их веса.
 * Оптимизирован для большого числа элементов.
 *
 * @param array $values индексный массив элементов
 * @param array $weights индексный массив соответствующих весов
 * @param array $lookup отсортированный массив для поиска
 * @param int $total_weight сумма всех весов
 * @return mixed выбранный элемент
 */
use rand::Rng;
pub fn weighted_random(
    weights: &Vec<u32>,
    lookup: &mut Vec<u32>,
    total_weight: u32,
) -> usize {
    if lookup.len() == 0 || total_weight == 0 {
        calc_lookups(&weights, lookup);
    }

    let r: u32 = rand::thread_rng().gen_range(1..=total_weight);
    binary_search(r, lookup)
}

/**
 * Создание массива используемого в бинарном поиске
 *
 * @param array $values
 * @param array $weights
 * @return array
 */
pub fn calc_lookups<'a>(weights: &'a Vec<u32>, lookup: &'a mut Vec<u32>) -> u32 {
    let mut total_weight: u32 = 0;

    for w in weights {
        total_weight += *w;
        lookup.push(total_weight);
    }

    total_weight
}

/**
 * Ищет в массиве элемент по номеру и возвращает элемент если он найден.
 * В противном случае возвращает позицию, где он должен быть вставлен,
 * или count($haystack)-1, если $needle больше чем любой элемент в массиве.
 *
 * @param int $needle
 * @param array $haystack
 * @return int
 */
fn binary_search(needle: u32, haystack: &Vec<u32>) -> usize {
    let mut high: usize = haystack.len() - 1;
    let mut low = 0;
    let mut probe = 0;

    while low < high {
        probe = ((high + low) / 2) as usize;
// println!("l={} h={} p={} n={} hs={}", low, high, probe, needle, haystack[probe]);

        if haystack[probe] < needle {
            if probe < haystack.len() - 1{
                low = probe + 1;
            }else{
                low = probe;
            }
        } else if haystack[probe] > needle{
            if probe > 0 {
                high = probe -1;
            }else{
                high = probe;
            }
        } else {
            return probe;
        }
    }

    if low != high {
        return probe;
    } else {
        if haystack[low] >= needle {
            return low;
        } else {
            return low + 1;
        }
    }
}
