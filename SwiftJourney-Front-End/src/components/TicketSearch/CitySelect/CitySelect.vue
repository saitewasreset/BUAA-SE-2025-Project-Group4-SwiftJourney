<template>
    <div class="city_choose_wrap" :style="{ top: pos.top + 'px', left: pos.left + 'px' }">
        <div class="choose_right">
            <el-tabs v-model="activeTag" class="demo-tabs">
                <el-tab-pane label="热门" name="hotWelcomed">
                    <div class="city_name">
                        <p v-for="item in handleCityMap['hotWelcomed']" @click="handleCityClick(item)">
                            {{ item.cityName }}
                        </p>
                    </div>
                </el-tab-pane>
                <el-tab-pane label="ABCDEF" name="ABCDEF">
                    <div class="city_name_wrap">
                        <div class="city_name_box" v-for="c in 'ABCDEF'.split('')">
                            <div class="font-semibold">{{ c }}</div>
                            <div class="city_name">
                                <p v-for="item in partTwoByCharacter(c)" @click="handleCityClick(item)">
                                    {{ item.cityName }}
                                </p>
                            </div>
                        </div>
                    </div>
                </el-tab-pane>
                <el-tab-pane label="GHIJ" name="GHIJ">
                    <div class="city_name_wrap">
                        <div class="city_name_box" v-for="c in 'GHIJ'.split('')">
                            <div class="font-semibold">{{ c }}</div>
                            <div class="city_name">
                                <p v-for="item in partTwoByCharacter(c)" @click="handleCityClick(item)">
                                    {{ item.cityName }}
                                </p>
                            </div>
                        </div>
                    </div>
                </el-tab-pane>
                <el-tab-pane label="KLMN" name="KLMN">
                    <div class="city_name_wrap">
                        <div class="city_name_box" v-for="c in 'KLMN'.split('')">
                            <div class="font-semibold">{{ c }}</div>
                            <div class="city_name">
                                <p v-for="item in partTwoByCharacter(c)" @click="handleCityClick(item)">
                                    {{ item.cityName }}
                                </p>
                            </div>
                        </div>
                    </div>
                </el-tab-pane>
                <el-tab-pane label="PQRSTUVW" name="PQRSTUVW">
                    <div class="city_name_wrap">
                        <div class="city_name_box" v-for="c in 'PQRSTUVW'.split('')">
                            <div class="font-semibold">{{ c }}</div>
                            <div class="city_name">
                                <p v-for="item in partTwoByCharacter(c)" @click="handleCityClick(item)">
                                    {{ item.cityName }}
                                </p>
                            </div>
                        </div>
                    </div>
                </el-tab-pane>
                <el-tab-pane label="XYZ" name="XYZ">
                    <div class="city_name_wrap">
                        <div class="city_name_box" v-for="c in 'XYZ'.split('')">
                            <div class="font-semibold">{{ c }}</div>
                            <div class="city_name">
                                <p v-for="item in partTwoByCharacter(c)" @click="handleCityClick(item)">
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
    import { onMounted, ref, reactive } from 'vue'
    import city from './city.json'

    const { el } = defineProps<{
        el?: HTMLElement,
    }>()

    const activeTag = ref('hotWelcomed')

    const pos = reactive({
        top: 0,
        left: 0,
    })

    const emit = defineEmits(['handleCityClick']);

    function calcModalPosition() {
        if (el) {
            pos.top = el.getBoundingClientRect().height + el.offsetTop + 6
            pos.left = el.offsetLeft
        }
    }

    const handleCityMap = ref<{ [key: string]: any }>({
        hotWelcomed: [
            {
                cityName: '北京',
                pinYin: 'BEIJING',
            },
        ],
    })

    const cityList = ref<any[]>([])

    function handleCityClick(item: string) {
        emit('handleCityClick', item);
    }

    function partTwoByCharacter(c: string) {
        c = c.toUpperCase()
        return cityList.value.filter((v: any) => {
            return v.pinYin.charAt(0).toUpperCase() === c
        })
    }

    onMounted(() => {
        calcModalPosition();
        cityList.value = Object.values(city).flat()
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
</style>
