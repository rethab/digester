<template>
  <div>
    <v-row justify="center" align="center">
      <v-col class="text-center">
        <h1 class="display-2 font-weight-medium">This does not exist :(</h1>
        <v-img
          class="mt-5"
          style="display:block; margin-left: auto; margin-right:auto"
          width="80%"
          :src="require('@/assets/tree_swing.svg')"
        ></v-img>
        <p class="mt-5">
          Please go
          <v-btn @click="$router.go(-1)" class="px-1 mr-1" color="primary" small>back</v-btn>or
          <v-btn to="/" class="px-1" color="primary" small>home</v-btn>
        </p>
      </v-col>
    </v-row>
  </div>
</template>