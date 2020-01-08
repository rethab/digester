<template>
  <v-card color="secondary" class="lighten-4">
    <div>
      <v-card-title>
        {{value.channelName}}
        <v-icon small class="pl-1">{{ githubIcon }}</v-icon>
      </v-card-title>
      <v-card-subtitle v-if="!editing">
        <v-icon small>{{ calendarIcon }}</v-icon>
        {{value | showFrequency}}
        <v-icon @click="editing = true" small class="ml-3">{{ pencilIcon }}</v-icon>
      </v-card-subtitle>
      <v-card-subtitle v-else>
        <FrequencySelection v-model="value" />
        <v-card-actions>
          <v-spacer></v-spacer>
          <v-btn @click="editing = false" color="secondary" outlined>Cancel</v-btn>
          <v-btn @click.stop="save" class="primary">Save</v-btn>
        </v-card-actions>
      </v-card-subtitle>
    </div>
  </v-card>
</template>

<script>
import FrequencySelection from "@/components/subs/FrequencySelection.vue";
import { mdiCalendar, mdiGithubCircle, mdiPencilOutline } from "@mdi/js";
export default {
  components: {
    FrequencySelection
  },
  props: {
    value: {
      type: Object,
      required: true
    }
  },
  data() {
    return {
      editing: false,

      calendarIcon: mdiCalendar,
      githubIcon: mdiGithubCircle,
      pencilIcon: mdiPencilOutline
    };
  },
  methods: {
    save() {
      this.$store
        .dispatch("updateSubscription", {
          id: this.value.id,
          frequency: this.value.frequency,
          day: this.value.day,
          time: this.value.time
        })
        .then(() => {
          this.editing = false;
        });
    }
  },
  filters: {
    showFrequency: function(sub) {
      var fmt = "every ";
      if (sub.frequency == "Daily") fmt += "day";
      else fmt += sub.day;
      fmt += " at ";
      fmt += sub.time.substring(0, 5); // HH:MM
      return fmt;
    }
  }
};
</script>