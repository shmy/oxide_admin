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
    DEPARTMENT: {
      READ: 500,
      CREATE: 501,
      UPDATE: 502,
      DELETE: 503,
    },
    CACHE: {
      READ: 600,
      DELETE: 601,
    },
    ACCESS_LOG: {
      READ: 700,
      ACCESS_LOG: {
        READ: 1000,
        CREATE: 1001,
        UPDATE: 1002,
        DELETE: 1003,
      },
    },
  },
};
