import {defineStore} from 'pinia';
import {Ref, ref} from 'vue';


export const useApiStore = defineStore('api', () => {
  const apiKey: Ref<string> = ref('')

  return {
    apiKey
  }
}, {
  persist: true
});
