<template>
    <div class="city_choose_wrap" :style="{ top: pos.top + 'px', left: pos.left + 'px' }">
        <div class="choose_right">
            <el-tabs v-model="activeTag" class="demo-tabs">
                <el-tab-pane label="推荐" name="hotWelcomed">
                    <div class="city_name" style="display: flex; flex-wrap: wrap; gap: 10px;">
                        <p class="suggestion" v-for="item in props.input.slice(0, 20)" :key="item" @click="handleCityClick(item)">
                            {{ item }}
                        </p>
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
import { reactive, onMounted, ref } from 'vue';
import { useGeneralStore } from '@/stores/general';

const generalStore = useGeneralStore();

const props = defineProps({
  input: {
    type: Object, // 因为input是一个ref对象
    required: true
  },
  el: {
    type: HTMLElement,
    default: null,
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

const cityList = ref<any[]>([])

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

onMounted(() => {
    calcModalPosition();
    cityList.value = generalStore.CityPinYinList;
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
</style>
