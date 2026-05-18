#!/usr/bin/env python3
"""
Тест производительности симуляции AEVUM
Запускает симуляцию с различным количеством сфер
"""

import sys
import time

# Импортировать модуль (после компиляции)
try:
    import sim_bridge
    print("✓ Модуль sim_bridge успешно импортирован")
except ImportError as e:
    print(f"✗ Ошибка импорта: {e}")
    print("Убедитесь, что модуль скомпилирован: maturin develop")
    sys.exit(1)


def test_basic():
    """Базовый тест симуляции"""
    print("\n=== Базовый тест ===")
    
    sim = sim_bridge.Simulation()
    print(f"Начальное количество сущностей: {sim.entity_count()}")
    
    # Спавн пола
    sim.spawn_ground(0.0)
    print(f"После спавна пола: {sim.entity_count()}")
    
    # Спавн 100 сфер
    sim.spawn_sphere_grid(100, 2.5, 1.0, 1.0)
    print(f"После спавна 100 сфер: {sim.entity_count()}")
    
    # Запустить 100 шагов
    start = time.perf_counter()
    sim.run(100)
    elapsed = time.perf_counter() - start
    
    print(f"100 шагов выполнено за {elapsed*1000:.2f} мс")
    print(f"Среднее время шага: {elapsed*10:.2f} мс")
    print(f"Количество сущностей: {sim.entity_count()}")
    

def benchmark(counts=[100, 1000, 10000]):
    """Бенчмарк производительности"""
    print("\n=== Бенчмарк производительности ===")
    print(f"{'Сфер':<10} {'Время (мс)':<15} {'Шаг (мкс)':<15}")
    print("-" * 40)
    
    for count in counts:
        sim = sim_bridge.Simulation()
        sim.spawn_ground(0.0)
        sim.spawn_sphere_grid(count, 2.5, 1.0, 1.0)
        
        start = time.perf_counter()
        sim.run(100)
        elapsed = time.perf_counter() - start
        
        ms_per_step = elapsed * 10  # 100 шагов
        us_per_step = elapsed * 10000  # 100 шагов
        
        print(f"{count:<10} {ms_per_step:<15.2f} {us_per_step:<15.2f}")


if __name__ == "__main__":
    test_basic()
    benchmark([100, 1000, 5000])
    
    print("\n✓ Тесты завершены!")
