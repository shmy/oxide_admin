import { lazy, Suspense } from "react";

const LazyReactTest = lazy(() => import("../components/react-test"));
const LazyByteDisplay = lazy(() => import("../components/byte-display"));

export const registerComponents = (amisLib: any) => {

    amisLib.Renderer({
        type: "react-test",
        autoVar: true,
    })(withSuspense(LazyReactTest));

    amisLib.FormItem({
        type: "byte-display",
        autoVar: true,
    })(withSuspense(LazyByteDisplay));

};

const withSuspense = (Comp: any) => {
    return (props: any) => {
        return (
            <Suspense fallback={<span>Loading...</span>}>
                <Comp {...props} />
            </Suspense>
        );
    };
};
