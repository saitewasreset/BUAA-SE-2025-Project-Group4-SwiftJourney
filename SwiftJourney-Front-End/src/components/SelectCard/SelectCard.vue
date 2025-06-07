<template>
    <div class="city_choose_wrap" :style="{ top: pos.top + 'px', left: pos.left + 'px' }">
        <div class="choose_right">
            <el-tabs v-model="activeTag" class="demo-tabs">
                <el-tab-pane label="搜索结果" name="hotWelcomed">
                    <!-- 有搜索结果时显示 -->
                    <div v-if="updateSuggestions && updateSuggestions.length > 0" class="city_name" style="display: flex; flex-wrap: wrap; gap: 10px;">
                        <p class="suggestion" v-for="item in updateSuggestions" :key="item" @click="handleCityClick(item)">
                            {{ item }}
                        </p>
                    </div>
                    <!-- 有输入但无搜索结果时显示 -->
                    <div v-else-if="userInput.trim() !== ''" class="no-results">
                        <div class="no-results-icon">
                            <el-icon size="48"><Search /></el-icon>
                        </div>
                        <h3 class="no-results-title">暂无搜索结果</h3>
                        <p class="no-results-subtitle">
                            未找到包含 "<span class="search-keyword">{{ userInput }}</span>" 的{{ getSearchTypeText() }}
                        </p>
                        <p class="no-results-tip">请尝试使用其他关键词或检查拼写</p>
                    </div>
                    <!-- 无输入时显示提示 -->
                    <div v-else class="search-hint">
                        <div class="search-hint-icon">
                            <el-icon size="40"><Edit /></el-icon>
                        </div>
                        <h4 class="search-hint-title">请输入{{ getSearchTypeText() }}名称</h4>
                        <p class="search-hint-subtitle">支持中文名称或拼音搜索</p>
                    </div>
                </el-tab-pane>
                <el-tab-pane label="ABCDEF" name="ABCDEF">
                    <div class="city_name_wrap">
                        <div class="city_name_box" v-for="c in 'ABCDEF'.split('')" :key="c">
                            <div class="font-semibold">{{ c }}</div>
                            <div class="city_name">
                                <p v-for="(item, index) in partTwoByCharacter(c)" :key="index" @click="handleCityClick(item.cityName)">
                                    {{ item.cityName }}
                                </p>
                            </div>
                        </div>
                    </div>
                </el-tab-pane>
                <el-tab-pane label="GHIJ" name="GHIJ">
                    <div class="city_name_wrap">
                        <div class="city_name_box" v-for="c in 'GHIJ'.split('')" :key="c">
                            <div class="font-semibold">{{ c }}</div>
                            <div class="city_name">
                                <p v-for="(item, index) in partTwoByCharacter(c)" :key="index" @click="handleCityClick(item.cityName)">
                                    {{ item.cityName }}
                                </p>
                            </div>
                        </div>
                    </div>
                </el-tab-pane>
                <el-tab-pane label="KLMN" name="KLMN">
                    <div class="city_name_wrap">
                        <div class="city_name_box" v-for="c in 'KLMN'.split('')" :key="c">
                            <div class="font-semibold">{{ c }}</div>
                            <div class="city_name">
                                <p v-for="(item, index) in partTwoByCharacter(c)" :key="index" @click="handleCityClick(item.cityName)">
                                    {{ item.cityName }}
                                </p>
                            </div>
                        </div>
                    </div>
                </el-tab-pane>
                <el-tab-pane label="PQRSTUVW" name="PQRSTUVW">
                    <div class="city_name_wrap">
                        <div class="city_name_box" v-for="c in 'PQRSTUVW'.split('')" :key="c">
                            <div class="font-semibold">{{ c }}</div>
                            <div class="city_name">
                                <p v-for="(item, index) in partTwoByCharacter(c)" :key="index" @click="handleCityClick(item.cityName)">
                                    {{ item.cityName }}
                                </p>
                            </div>
                        </div>
                    </div>
                </el-tab-pane>
                <el-tab-pane label="XYZ" name="XYZ">
                    <div class="city_name_wrap">
                        <div class="city_name_box" v-for="c in 'XYZ'.split('')" :key="c">
                            <div class="font-semibold">{{ c }}</div>
                            <div class="city_name">
                                <p v-for="(item, index) in partTwoByCharacter(c)" :key="index" @click="handleCityClick(item.cityName)">
                                    {{ item.cityName }}
                                </p>
                            </div>
                        </div>
                    </div>
                </el-tab-pane>
            </el-tabs>
        </div>
    </div>
</template>

<script setup lang="ts">
import { reactive, onMounted, ref, computed } from 'vue';
import { useGeneralStore } from '@/stores/general';
import { Search, Edit } from '@element-plus/icons-vue';
import { pinyin } from 'pinyin-pro'

const generalStore = useGeneralStore();

const props = defineProps({
    input: {
        type: String,
        required: true
    },
    el: {
        type: HTMLElement,
        default: null,
    },
    type: {
        type: String, //city | station | both
        default: 'both',
    }
});

const pos = reactive({
    top: 0,
    left: 0,
})

const activeTag = ref('hotWelcomed')
const emit = defineEmits(['handleCityClick']);

function calcModalPosition() {
    if (props.el) {
        pos.top = props.el.getBoundingClientRect().height + props.el.offsetTop + 20
        pos.left = props.el.offsetLeft
    }
}

// 获取搜索类型文本
function getSearchTypeText() {
    switch (props.type) {
        case 'city':
            return '城市';
        case 'station':
            return '车站';
        case 'both':
        default:
            return '城市或车站';
    }
}

const cityList = computed(() =>{ 
    if(props.type == 'city') {
        return generalStore.CityPinYinList;
    } else if (props.type == 'station') {
        return generalStore.StationPinYinList;
    } else {
        return generalStore.BothPinYinList;
    }
});

function handleCityClick(item: string) {
    emit('handleCityClick', item);
}

function partTwoByCharacter(c: string) {
    c = c.toUpperCase()
    let result: {cityName: string, pinYin: string}[] = [];
    cityList.value.forEach((value) => {
        if(value.pinYin.charAt(0).toUpperCase() === c) {
            result.push(value);
        }
    });
    return result;
}

const userInput = computed(() => props.input);

// 优化后的搜索建议
const updateSuggestions = computed(() => {
    const input = userInput.value.trim();

    if(input === '') {
        return [];
    }

    const searchResults: Array<{name: string, score: number, type: string}> = [];

    // 1. 精确匹配（最高优先级）
    exactMatch(input, searchResults);
    
    // 2. 前缀匹配
    prefixMatch(input, searchResults);
    
    // 3. 拼音匹配
    pinyinMatch(input, searchResults);
    
    // 4. 模糊匹配
    fuzzyMatch(input, searchResults);
    
    // 5. 包含匹配
    containsMatch(input, searchResults);

    // 按分数排序并去重
    const uniqueResults = Array.from(new Map(
        searchResults.map(item => [item.name, item])
    ).values());
    
    return uniqueResults
        .sort((a, b) => b.score - a.score) // 按分数降序排列
        .slice(0, 20)
        .map(item => item.name);
});

// 1. 精确匹配
function exactMatch(input: string, results: Array<{name: string, score: number, type: string}>) {
    const searchData = getSearchData();
    
    searchData.forEach(item => {
        if (item.name === input) {
            results.push({
                name: item.name,
                score: 1000, // 最高分数
                type: item.type
            });
        }
    });
}

// 2. 前缀匹配
function prefixMatch(input: string, results: Array<{name: string, score: number, type: string}>) {
    const searchData = getSearchData();
    
    searchData.forEach(item => {
        // 中文前缀匹配
        if (item.name.startsWith(input)) {
            results.push({
                name: item.name,
                score: 900,
                type: item.type
            });
        }
        
        // 拼音前缀匹配
        const fullPinyin = pinyin(item.name, { toneType: 'none', type: 'array' }).join('');
        const firstLetters = pinyin(item.name, { pattern: 'first', toneType: 'none', type: 'array' }).join('');
        
        if (fullPinyin.toLowerCase().startsWith(input.toLowerCase()) ||
            firstLetters.toLowerCase().startsWith(input.toLowerCase())) {
            results.push({
                name: item.name,
                score: 850,
                type: item.type
            });
        }
    });
}

// 3. 增强的拼音匹配
function pinyinMatch(input: string, results: Array<{name: string, score: number, type: string}>) {
    const searchData = getSearchData();
    
    searchData.forEach(item => {
        const score = calculatePinyinScore(input, item.name);
        if (score > 0) {
            results.push({
                name: item.name,
                score: score,
                type: item.type
            });
        }
    });
}

// 4. 模糊匹配
function fuzzyMatch(input: string, results: Array<{name: string, score: number, type: string}>) {
    const searchData = getSearchData();
    
    searchData.forEach(item => {
        const score = calculateFuzzyScore(input, item.name);
        if (score > 0.6) { // 相似度阈值
            results.push({
                name: item.name,
                score: Math.floor(score * 600), // 转换为分数
                type: item.type
            });
        }
    });
}

// 5. 包含匹配
function containsMatch(input: string, results: Array<{name: string, score: number, type: string}>) {
    const searchData = getSearchData();
    
    searchData.forEach(item => {
        if (item.name.includes(input)) {
            results.push({
                name: item.name,
                score: 500,
                type: item.type
            });
        }
    });
}

// 获取搜索数据
function getSearchData(): Array<{name: string, type: string}> {
    const data: Array<{name: string, type: string}> = [];
    
    if (props.type === 'city' || props.type === 'both') {
        generalStore.CityPinYinList?.forEach(item => {
            data.push({ name: item.cityName, type: 'city' });
        });
    }
    
    if (props.type === 'station' || props.type === 'both') {
        generalStore.StationPinYinList?.forEach(item => {
            data.push({ name: item.cityName, type: 'station' });
        });
    }
    
    return data;
}

// 计算拼音匹配分数
function calculatePinyinScore(input: string, cityName: string): number {
    let score = 0;
    const inputLower = input.toLowerCase();
    
    // 全拼匹配
    const fullPinyin = pinyin(cityName, { toneType: 'none', type: 'array' }).join('').toLowerCase();
    if (fullPinyin.includes(inputLower)) {
        score += 700;
        if (fullPinyin.startsWith(inputLower)) {
            score += 100; // 前缀加分
        }
    }
    
    // 首字母匹配
    const firstLetters = pinyin(cityName, { pattern: 'first', toneType: 'none', type: 'array' }).join('').toLowerCase();
    if (firstLetters.includes(inputLower)) {
        score += 600;
        if (firstLetters.startsWith(inputLower)) {
            score += 100; // 前缀加分
        }
    }
    
    // 分段拼音匹配
    const pinyinArray = pinyin(cityName, { toneType: 'none', type: 'array' });
    for (let i = 0; i < pinyinArray.length; i++) {
        if (pinyinArray[i].toLowerCase().startsWith(inputLower)) {
            score += 650;
            break;
        }
    }
    
    // 混合匹配（支持 "bj" 匹配 "北京"）
    if (isValidPinyinAbbreviation(inputLower, cityName)) {
        score += 750;
    }
    
    return score;
}

// 验证是否为有效的拼音缩写
function isValidPinyinAbbreviation(input: string, cityName: string): boolean {
    const firstLetters = pinyin(cityName, { pattern: 'first', toneType: 'none', type: 'array' });
    const inputChars = input.split('');
    
    if (inputChars.length > firstLetters.length) return false;
    
    for (let i = 0; i < inputChars.length; i++) {
        if (firstLetters[i]?.toLowerCase() !== inputChars[i].toLowerCase()) {
            return false;
        }
    }
    
    return true;
}

// 计算模糊匹配分数（编辑距离算法）
function calculateFuzzyScore(input: string, target: string): number {
    const distance = levenshteinDistance(input.toLowerCase(), target.toLowerCase());
    const maxLength = Math.max(input.length, target.length);
    return 1 - (distance / maxLength);
}

// 计算编辑距离
function levenshteinDistance(str1: string, str2: string): number {
    const matrix = Array(str2.length + 1).fill(null).map(() => Array(str1.length + 1).fill(null));
    
    for (let i = 0; i <= str1.length; i++) {
        matrix[0][i] = i;
    }
    
    for (let j = 0; j <= str2.length; j++) {
        matrix[j][0] = j;
    }
    
    for (let j = 1; j <= str2.length; j++) {
        for (let i = 1; i <= str1.length; i++) {
            const indicator = str1[i - 1] === str2[j - 1] ? 0 : 1;
            matrix[j][i] = Math.min(
                matrix[j][i - 1] + 1, // deletion
                matrix[j - 1][i] + 1, // insertion
                matrix[j - 1][i - 1] + indicator // substitution
            );
        }
    }
    
    return matrix[str2.length][str1.length];
}

onMounted(() => {
    calcModalPosition();
    generalStore.init();
})
</script>

<style lang="css" scoped>
.city_choose_wrap {
    position: absolute;
    background: #fff;
    box-shadow: 0px 0px 12px #ccc;
    border-radius: 8px;
    overflow: hidden;
    display: flex;
    height: 300px;
    width: auto;
    max-width: 477px;
    z-index: 999;

    .choose_right {
        background: #fff;
        padding: 4px 12px;
        height: full;
        overflow-y: auto;

        p {
            cursor: pointer;

            &:hover {
                color: #3d6cfe;
            }
        }

        .city_name_wrap {
            height: 224px;
            overflow-y: auto;

            .city_name_box {
                display: flex;
                gap: 20px;

                .font-semibold {
                    font-weight: 600;
                }

                .city_name {
                    display: flex;
                    flex-wrap: wrap;
                    gap: 8px;
                    font-size: 14px;
                    line-height: 20px;
                    color: #374151;
                    margin-bottom: 16px;
                }
            }
        }
    }

    .slide_active {
        background: #3d6cfe;
        color: #fff;
    }
}

.suggestion {
    font-size: 16px;
    color: #374151;
    margin-bottom: 0;
    cursor: pointer;

    &:hover {
        color: #3d6cfe;
    }
}

/* 新增样式：无搜索结果 */
.no-results {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 40px 20px;
    text-align: center;
    height: 200px;
}

.no-results-icon {
    margin-bottom: 16px;
    color: #9ca3af;
}

.no-results-title {
    font-size: 18px;
    font-weight: 600;
    color: #374151;
    margin: 0 0 8px 0;
}

.no-results-subtitle {
    font-size: 14px;
    color: #6b7280;
    margin: 0 0 8px 0;
    line-height: 1.5;
}

.search-keyword {
    color: #3d6cfe;
    font-weight: 600;
}

.no-results-tip {
    font-size: 12px;
    color: #9ca3af;
    margin: 0;
    line-height: 1.4;
}

/* 新增样式：搜索提示 */
.search-hint {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 40px 20px;
    text-align: center;
    height: 200px;
}

.search-hint-icon {
    margin-bottom: 16px;
    color: #d1d5db;
}

.search-hint-title {
    font-size: 16px;
    font-weight: 600;
    color: #6b7280;
    margin: 0 0 8px 0;
}

.search-hint-subtitle {
    font-size: 14px;
    color: #9ca3af;
    margin: 0;
    line-height: 1.5;
}

/* 响应式设计 */
@media (max-width: 480px) {
    .city_choose_wrap {
        max-width: 320px;
    }
    
    .no-results,
    .search-hint {
        padding: 30px 15px;
        height: 180px;
    }
    
    .no-results-title {
        font-size: 16px;
    }
    
    .search-hint-title {
        font-size: 14px;
    }
    
    .no-results-subtitle,
    .search-hint-subtitle {
        font-size: 13px;
    }
}
</style>
