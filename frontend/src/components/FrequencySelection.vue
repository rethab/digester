<template>
  <v-row cols="12">
    <v-col cols="5">
      <v-select v-model="frequency" :items="frequencies" append-icon></v-select>
    </v-col>
    <v-col cols="3">
      <v-select v-model="day" :disabled="!isWeekly" :items="days" append-icon></v-select>
    </v-col>
    <v-col cols="4">
      <v-select v-model="hour" :items="hours" append-icon></v-select>
    </v-col>
  </v-row>
</template>

<script>
export default {
  data() {
    return {
      frequencies: ["weekly", "daily"],
      frequency: "weekly",
      days: ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"],
      day: "Sat",
      hours: [],
      hour: "09:00"
    };
  },
  computed: {
    isWeekly() {
      return this.frequency === "weekly";
    }
  },
  mounted() {
    var hours = [];
    for (var h = 1; h < 25; h++) {
      var formatted = h < 10 ? "0" + h : "" + h;
      formatted = formatted + ":00";
      hours.push(formatted);
    }
    this.hours = hours;
  }
};
</script>