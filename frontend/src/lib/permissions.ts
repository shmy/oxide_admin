export const PERMISSIONS = {
  SYSTEM: {
    USER: {
      READ: 100,
      CREATE: 101,
      UPDATE: 102,
      DELETE: 103,
      ENABLE: 104,
      DISABLE: 105,
      UPDATE_PASSWORD: 106,
    },
    ROLE: {
      READ: 200,
      CREATE: 201,
      UPDATE: 202,
      DELETE: 203,
      ENABLE: 204,
      DISABLE: 205,
    },
    FILE: {
      READ: 300,
      UPLOAD: 301,
      DOWNLOAD: 302,
    },
    SCHED: {
      READ: 400,
      DELETE: 401,
    },
    BGWORKER: {
      READ: 500,
    },
    DEPARTMENT: {
      READ: 600,
      CREATE: 601,
      UPDATE: 602,
      DELETE: 603,
    },
    CACHE: {
      READ: 700,
      DELETE: 701,
    },
    ACCESS_LOG: {
      READ: 800,
    },
  },
};
