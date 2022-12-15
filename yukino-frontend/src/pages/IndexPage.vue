<template>
  <q-page class="q-mx-md">
    <h6>Save a valid Premiumize Api Key for redirection</h6>
    <q-input v-model="apiInput" label="Premiumize API Key"></q-input>
    <q-btn color="primary" class="q-mt-sm" @click="saveKey()">save</q-btn>
  </q-page>
</template>

<script setup lang="ts">
import { api } from 'src/boot/axios';
import { useApiStore } from 'src/stores/Api';
import { onMounted, ref } from 'vue';
import { useQuasar } from 'quasar';
import { data } from 'browserslist';

const $q = useQuasar();
const apiStore = useApiStore();
const apiInput = ref(apiStore.apiKey);

function saveKey() {
  apiStore.apiKey = apiInput.value;
  createTransferAndRedirect();
}

onMounted(() => {
  createTransferAndRedirect();
});

function createTransferAndRedirect() {
  const url = window.location.href;
  const pattern = /\/\?r=(.*)/;
  let redirect = '';
  try {
    // @ts-ignore
    redirect = url.match(pattern)[1];
  } catch (e) {
    if (e instanceof TypeError) {
      $q.notify({
        message:
          'Magnetlink Missing.',
        color: 'negative'
      });
    }
    return;
  }
  const data = new FormData();
  data.append('src', redirect);
  console.log(redirect);
  api
    .post(`transfer/create?apikey=${apiStore.apiKey}`, data, {
      headers: {
        'Content-Type': 'multipart/form-data',
        accept: 'application/json'
      }
    })
    .then((r) => {
      console.log(r);
      if (
        r.data.status !== 'error' ||
        r.data.message === 'You already added this job.'
      ) {
        window.location.replace('https://www.premiumize.me/transfers');
      } else {
        $q.notify({
          message:
            'Something went wrong while adding transfer. Check Apikey or Magnetlink.',
          color: 'negative'
        });
      }
    });
}
</script>
