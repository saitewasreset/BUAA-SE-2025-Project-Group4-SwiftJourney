import { ref, computed } from 'vue'
import { defineStore } from 'pinia'

export const useCounterStore = defineStore('counter', () => {
  const count = ref(0)
  const doubleCount = computed(() => count.value * 2)
  function increment() {
    count.value++
  }

  return { count, doubleCount, increment }
});

export const useUserStore = defineStore('user', {
    state: () => ({
        personalId: '',
        name: '',
        identityCardId: '',
        preferredSeatLocation: 'A' as 'A' | 'B' | 'C' | 'D' | 'F',
        token: '',
        default: false,
    }),
    getters: {

    },
    actions: {
        setPreferredSeatLocation(location: 'A' | 'B' | 'C' | 'D' | 'F') {
            this.preferredSeatLocation = location;
        },
        setUserDetails(personalId: string, name: string, identityCardId: string) {
            this.personalId = personalId;
            this.name = name;
            this.identityCardId = identityCardId;
        }
    }
})
