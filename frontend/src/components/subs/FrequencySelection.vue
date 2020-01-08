<template>
  <v-row cols="12">
    <v-col cols="5">
      <v-select
        v-model="value.frequency"
        :items="frequencies"
        @change="frequencyChanged"
        append-icon
      ></v-select>
    </v-col>
    <v-col cols="3">
      <v-select v-model="value.day" :disabled="!isWeekly" :items="days" append-icon></v-select>
    </v-col>
    <v-col cols="4">
      <v-select v-model="value.time" :items="times" append-icon></v-select>
    </v-col>
  </v-row>
</template>

<script>
export default {
  props: {
    value: {
      type: Object,
      required: true
    }
  },
  data() {
    return {
      isWeekly: this.value.frequency === "Weekly",

      frequencies: ["Weekly", "Daily"],
      days: ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"],
      times: []
    };
  },
  methods: {
    frequencyChanged(newVal) {
      this.isWeekly = newVal === "Weekly";
    }
  },
  mounted() {
    var times = [];
    for (var h = 1; h < 25; h++) {
      var formatted = h < 10 ? "0" + h : "" + h;
      var text = formatted + ":00"; // HH:MM
      // backend also wants seconds and millis
      var value = text + ":00"; // HH:MM:SS
      times.push({
        text: text,
        value: value
      });
    }
    this.times = times;
  }
};
</script>