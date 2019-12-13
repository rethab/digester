<template>
  <div>
    <v-container fluid>
      <v-row dense>
        <v-col v-for="subscription in subscriptions" :key="subscription.name" cols="12">
          <v-card color="blue" class="lighten-4">
            <div>
              <v-card-title>
                {{subscription.name}}
                <v-icon small class="pl-1">mdi-github-circle</v-icon>
              </v-card-title>
              <v-card-subtitle>
                <v-icon small>mdi-calendar</v-icon>
                {{subscription.frequency}}
              </v-card-subtitle>
            </div>
          </v-card>
        </v-col>
      </v-row>
    </v-container>
  </div>
</template>

<script>
export default {
  data() {
    return {
      subscriptions: this.$store.state.subscriptions
    };
  }
};
</script>
