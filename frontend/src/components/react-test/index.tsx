import { Button, Segmented } from "antd";
import { useEffect } from "react";

const ReactTest = () => {
  useEffect(() => {
    console.log("ReactTest onMounted");
    return () => {
      console.log("ReactTest onUnmounted");
    };
  }, []);

  return (
    <div>
      <Button
        type="primary"
        onClick={() => {
          console.log("click");
        }}
      >
        我是Antd的Button
      </Button>
      <br />
      <h3>我是Antd的Segmented</h3>
      <Segmented<string>
        options={["Daily", "Weekly", "Monthly", "Quarterly", "Yearly"]}
        onChange={(value) => {
          console.log(value); // string
        }}
      />
    </div>
  );
};

export default ReactTest;
