import random
import csv

# 精简后的中国发达城市列表（主要为一线和强二线城市）
developed_cities = [
    "北京市", "上海市", "天津市", "重庆市",
    "广州市", "深圳市", "东莞市", "佛山市", "珠海市",
    "南京市", "苏州市", "无锡市", "常州市", "南通市",
    "杭州市", "宁波市", "温州市", "嘉兴市", "绍兴市",
    "济南市", "青岛市", "烟台市", "威海市",
    "成都市", "武汉市",
    "福州市", "厦门市", "泉州市",
    "长沙市", "郑州市",
    "沈阳市", "大连市", "西安市",
    "石家庄市", "唐山市",
    "合肥市", "哈尔滨市", "长春市",
    "昆明市", "贵阳市", "南宁市", "太原市", "南昌市"
]

def generate_city_pairs(num_pairs):
    """生成指定数量的城市对"""
    pairs = []
    for _ in range(num_pairs):
        # 随机选择两个不同的城市
        city1, city2 = random.sample(developed_cities, 2)
        pairs.append({
            'departureCity': city1,
            'arrivalCity': city2
        })
    return pairs

def save_to_csv(pairs, filename='cityPairs.csv'):
    """将城市对保存到CSV文件"""
    with open(filename, 'w', newline='', encoding='utf-8') as csvfile:
        fieldnames = ['departureCity', 'arrivalCity']
        writer = csv.DictWriter(csvfile, fieldnames=fieldnames)
        
        writer.writeheader()
        for pair in pairs:
            writer.writerow(pair)

def main():
    # 获取用户输入的生成数量
    try:
        num_pairs = int(input("请输入要生成的城市对数量: "))
        if num_pairs <= 0:
            print("请输入一个正整数。")
            return
    except ValueError:
        print("请输入有效的数字。")
        return
    
    # 生成城市对
    city_pairs = generate_city_pairs(num_pairs)
    
    # 保存到CSV文件
    save_to_csv(city_pairs)
    print(f"已生成 {num_pairs} 个城市对并保存到 cityPairs.csv")

if __name__ == "__main__":
    main()