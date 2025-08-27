import { Button, FloatButton } from "antd";
import { useEffect } from "react";

export const ReactTest = () => {
  useEffect(() => {
    console.log("useEffect 1234");
    return () => {
      console.log("onMounted");
    };
  }, []);

  return (
    <div>
      <button
        type="button"
        onClick={() => {
          console.log("click");
        }}
        className="react-test"
      >
        ReactTest
      </button>
      <Button
        type="primary"
        onClick={() => {
          console.log("click");
        }}
      >
        {" "}
        1
      </Button>
      <FloatButton />
    </div>
  );
};
